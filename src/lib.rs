use libc::c_char;
use std::ffi::{
    CStr,
    CString,
    c_void,
};
use std::{fs, ptr};
use toml_edit::{
    Document,
    Table,
    Item,
    Value,
    Formatted,
};
use std::error::Error;
use std::str::FromStr;

// declare an enum for the package manager: nipm or vipm.
enum PackageManager {
    NIPM,
    VIPM,
}

fn parse_package_manager(package_manager: &str) -> Result<PackageManager, &'static str> {
    match package_manager {
        "nipm" => Ok(PackageManager::NIPM),
        "vipm" => Ok(PackageManager::VIPM),
        _ => Err("Invalid package_manager argument")
    }
}

fn toml_set_package_attribute_string(
    toml_str: String,
    package_manager: PackageManager,
    package_name: &str,
    attribute_name: &str,
    attribute_value: &str,
) -> Result<String, Box<dyn Error>> {
    const MGR_TABLE_NAME: [&str; 2] = ["nipm", "vipm"];
    const DEPS_SUBTABLE_NAME: &str = "dependencies";
    const VERSION_SUBKEY_NAME: &str = "version";

    let mgr_table_name = MGR_TABLE_NAME[package_manager as usize];

    let mut doc = toml_str.parse::<Document>()?;

    // we should initialize these tables if they don't yet exist
    if !doc.contains_table(mgr_table_name) {
        doc[mgr_table_name] = Item::Table(toml_edit::Table::new());
    }
    if !doc[mgr_table_name].as_table().unwrap().contains_table(DEPS_SUBTABLE_NAME) {
        doc[mgr_table_name][DEPS_SUBTABLE_NAME] = Item::Table(toml_edit::Table::new());
    }

    let mgr_table = doc[mgr_table_name].as_table_mut().ok_or("Table not found")?;
    let mgr_deps_table = mgr_table[DEPS_SUBTABLE_NAME].as_table_mut().ok_or("Dependencies table not found")?;

    let already_is_inline_table = mgr_deps_table.contains_key(package_name) &&
        mgr_deps_table[package_name].is_inline_table();

    let is_version_attribute = attribute_name == VERSION_SUBKEY_NAME;

    // use an inline table if the attribute_name is not the "version" or if it's already a subtable
    let use_inline_table = already_is_inline_table || !is_version_attribute;

    if use_inline_table {
        // if the package_name is not already a subtable, then make it one
        if !already_is_inline_table {
            if mgr_deps_table.contains_key(package_name) {
                let version_string = mgr_deps_table.remove(package_name).expect("Error reading version key").as_str().unwrap().to_string();
                mgr_deps_table[package_name][VERSION_SUBKEY_NAME] = Item::Value(Value::String(Formatted::new(version_string)));
            }
        }
        // add the attribute_name as a subkey
        mgr_deps_table[package_name][attribute_name] = Item::Value(Value::String(Formatted::new(attribute_value.to_string())));
    } else {
        // add the package version as a regular string key
        mgr_deps_table[package_name] = Item::Value(Value::String(Formatted::new(attribute_value.to_string())));
    }

    Ok(doc.to_string())
}

// a function that gets a string attribute of a package in a dependencies table.
// If the package is a simple string, it assumed to be the "version" attribute.
// If the package is a subtable, then the attribute is a subkey of the subtable.
fn toml_get_package_attribute_string(
    toml_str: String,
    package_manager: PackageManager,
    package_name: &str,
    attribute_name: &str,
) -> Result<String, Box<dyn Error>> {
    const MGR_TABLE_NAME: [&str; 2] = ["nipm", "vipm"];
    const DEPS_SUBTABLE_NAME: &str = "dependencies";
    const VERSION_SUBKEY_NAME: &str = "version";

    let mgr_table_name = MGR_TABLE_NAME[package_manager as usize];

    let mut doc = toml_str.parse::<Document>()?;

    let mgr_table = doc[mgr_table_name].as_table_mut().ok_or("Table not found")?;
    let mgr_deps_table = mgr_table[DEPS_SUBTABLE_NAME].as_table_mut().ok_or("Dependencies table not found")?;

    return if !mgr_deps_table.contains_key(package_name) {
       Err("Package not found")?
    } else if mgr_deps_table[package_name].is_inline_table() {
        if !mgr_deps_table[package_name].as_inline_table().unwrap().contains_key(attribute_name) {
            Err("Package attribute not found")?
        } else {
            Ok(mgr_deps_table[package_name][attribute_name].as_str().unwrap().to_string())
        }
    } else if attribute_name == VERSION_SUBKEY_NAME {
        Ok(mgr_deps_table[package_name].as_str().unwrap().to_string())
    } else {
        Err("Package attribute not found")?
    }
}


fn toml_remove_package_attribute(
    toml_str: String,
    package_manager: PackageManager,
    package_name: &str,
    attribute_name: &str,
) -> Result<String, Box<dyn Error>> {
    const MGR_TABLE_NAME: [&str; 2] = ["nipm", "vipm"];
    const DEPS_SUBTABLE_NAME: &str = "dependencies";

    let mgr_table_name = MGR_TABLE_NAME[package_manager as usize];

    let mut doc = toml_str.parse::<Document>()?;

    let mgr_table = doc[mgr_table_name].as_table_mut().ok_or("Table not found")?;
    let mgr_deps_table = mgr_table[DEPS_SUBTABLE_NAME].as_table_mut().ok_or("Dependencies table not found")?;

    return if !mgr_deps_table.contains_key(package_name) {
       Err("Package not found")?
    } else if mgr_deps_table[package_name].is_inline_table() {
        if !mgr_deps_table[package_name].as_inline_table().unwrap().contains_key(attribute_name) {
            Err("Package attribute not found")?
        } else {
            mgr_deps_table[package_name].as_inline_table_mut().unwrap().remove(attribute_name);
            Ok(doc.to_string())
        }
    } else {
        Err("Package attribute not found")?
    }
}


fn get_manager_dependencies_table<'a>(
    doc: &'a mut Document,
    package_manager: PackageManager,
) -> Result<&'a mut Table, Box<dyn Error>> {
    const MGR_TABLE_NAME: [&str; 2] = ["nipm", "vipm"];
    const DEPS_SUBTABLE_NAME: &str = "dependencies";

    let mgr_table_name = MGR_TABLE_NAME[package_manager as usize];
    let mgr_table = doc[mgr_table_name].as_table_mut().ok_or("Table not found")?;
    let mgr_deps_table = mgr_table[DEPS_SUBTABLE_NAME].as_table_mut().ok_or("Dependencies table not found")?;

    Ok(mgr_deps_table)
}


fn toml_remove_package(
    toml_str: String,
    package_manager: PackageManager,
    package_name: &str
) -> Result<String, Box<dyn Error>> {

    let mut doc = toml_str.parse::<Document>()?;

    let mgr_deps_table = match get_manager_dependencies_table(&mut doc, package_manager) {
        Ok(table) => table,
        Err(e) => return Err(e)
    };

    return if !mgr_deps_table.contains_key(package_name) {
       Err("Package not found")?
    } else {
        mgr_deps_table.remove(package_name);
        Ok(doc.to_string())
    }
}


// a function that lists the packages for a given package manager, these are the keys in the dependencies table.
fn toml_list_packages(
    toml_str: String,
    package_manager: PackageManager,
) -> Result<String, Box<dyn Error>> {
    const MGR_TABLE_NAME: [&str; 2] = ["nipm", "vipm"];
    const DEPS_SUBTABLE_NAME: &str = "dependencies";

    let mgr_table_name = MGR_TABLE_NAME[package_manager as usize];

    let mut doc = toml_str.parse::<Document>()?;

    // we should initialize these tables if they don't yet exist
    if !doc.contains_table(mgr_table_name) {
        doc[mgr_table_name] = Item::Table(toml_edit::Table::new());
    }
    if !doc[mgr_table_name].as_table().unwrap().contains_table(DEPS_SUBTABLE_NAME) {
        doc[mgr_table_name][DEPS_SUBTABLE_NAME] = Item::Table(toml_edit::Table::new());
    }

    let mgr_table = doc[mgr_table_name].as_table_mut().ok_or("Table not found")?;
    let mgr_deps_table = mgr_table[DEPS_SUBTABLE_NAME].as_table_mut().ok_or("Dependencies table not found")?;

    let mut package_list = String::new();
    for (package_name, _) in mgr_deps_table.iter() {
        package_list.push_str(&format!("{}\n", package_name));
    }

    Ok(package_list)
}

#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn set_package_attribute_string (
    file_path: *const c_char,
    package_manager: *const c_char,
    package_name: *const c_char,
    attribute_name: *const c_char,
    value_string: *const c_char,
) -> i32 {
let file_path = unsafe { CStr::from_ptr(file_path).to_string_lossy().into_owned() };
    let package_manager = unsafe { CStr::from_ptr(package_manager).to_string_lossy().into_owned() };
    let package_name = unsafe { CStr::from_ptr(package_name).to_string_lossy().into_owned() };
    let attribute_name = unsafe { CStr::from_ptr(attribute_name).to_string_lossy().into_owned() };
    let value_string = unsafe { CStr::from_ptr(value_string).to_string_lossy().into_owned() };

    let package_manager = match parse_package_manager(&package_manager) {
        Ok(manager) => manager,
        Err(_) => {
            println!("Invalid package_manager argument:
            {}", package_manager);
            return 1;
        }
    };

    let toml_str = match fs::read_to_string(&file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Unable to read file: {}", file_path);
            return 1;
        }
    };

    let modified_toml = match toml_set_package_attribute_string(
        toml_str,
        package_manager,
        &package_name,
        &attribute_name,
        &value_string,
    ) {
        Ok(toml) => toml,
        Err(e) => {
            println!("Error modifying toml file: {}", e);
            return 1;
        }
    };

    if let Err(_) = fs::write(&file_path, modified_toml) {
        println!("Unable to write file: {}", file_path);
        return 1;
    }

    return 0;
}

#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn get_package_attribute_string (
    file_path: *const c_char,
    package_manager: *const c_char,
    package_name: *const c_char,
    attribute_name: *const c_char,
) -> *mut c_char  {
    let file_path = unsafe { CStr::from_ptr(file_path).to_string_lossy().into_owned() };
    let package_manager = unsafe { CStr::from_ptr(package_manager).to_string_lossy().into_owned() };
    let package_name = unsafe { CStr::from_ptr(package_name).to_string_lossy().into_owned() };
    let attribute_name = unsafe { CStr::from_ptr(attribute_name).to_string_lossy().into_owned() };

    let package_manager = match parse_package_manager(&package_manager) {
        Ok(manager) => manager,
        Err(_) => {
            println!("Invalid package_manager argument:
            {}", package_manager);
            return CString::new("").unwrap().into_raw();
        }
    };

    let toml_str = match fs::read_to_string(&file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Unable to read file: {}", file_path);
            return CString::new("").unwrap().into_raw();
        }
    };

    let package_attribute_value = match toml_get_package_attribute_string(
        toml_str,
        package_manager,
        &package_name,
        &attribute_name,
    ) {
        Ok(value) => value,
        Err(e) => {
            println!("Error reading toml attribute: {}", e);
            return CString::new("").unwrap().into_raw();
        }
    };

    let raw_string = match CString::new(package_attribute_value).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;

}


#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn remove_package_attribute (
    file_path: *const c_char,
    package_manager: *const c_char,
    package_name: *const c_char,
    attribute_name: *const c_char,
) -> i32  {
    let file_path = unsafe { CStr::from_ptr(file_path).to_string_lossy().into_owned() };
    let package_manager = unsafe { CStr::from_ptr(package_manager).to_string_lossy().into_owned() };
    let package_name = unsafe { CStr::from_ptr(package_name).to_string_lossy().into_owned() };
    let attribute_name = unsafe { CStr::from_ptr(attribute_name).to_string_lossy().into_owned() };

    let package_manager = match parse_package_manager(&package_manager) {
        Ok(manager) => manager,
        Err(_) => {
            println!("Invalid package_manager argument:
            {}", package_manager);
            return 1
        }
    };

    let toml_str = match fs::read_to_string(&file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Unable to read file: {}", file_path);
            return 1
        }
    };

    let modified_toml = match toml_remove_package_attribute(
        toml_str,
        package_manager,
        &package_name,
        &attribute_name,
    ) {
        Ok(value) => value,
        Err(e) => {
            println!("Error removing toml attribute: {}", e);
            return 1;
        }
    };

    if let Err(_) = fs::write(&file_path, modified_toml) {
        println!("Unable to write file: {}", file_path);
        return 1;
    }

    return 0;

}


#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn remove_package (
    file_path: *const c_char,
    package_manager: *const c_char,
    package_name: *const c_char,
) -> i32  {
    let file_path = unsafe { CStr::from_ptr(file_path).to_string_lossy().into_owned() };
    let package_manager = unsafe { CStr::from_ptr(package_manager).to_string_lossy().into_owned() };
    let package_name = unsafe { CStr::from_ptr(package_name).to_string_lossy().into_owned() };

    let package_manager = match parse_package_manager(&package_manager) {
        Ok(manager) => manager,
        Err(_) => {
            println!("Invalid package_manager argument:
            {}", package_manager);
            return 1
        }
    };

    let toml_str = match fs::read_to_string(&file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Unable to read file: {}", file_path);
            return 1
        }
    };

    let modified_toml = match toml_remove_package(
        toml_str,
        package_manager,
        &package_name,
    ) {
        Ok(value) => value,
        Err(e) => {
            println!("Error removing package: {}", e);
            return 1;
        }
    };

    if let Err(_) = fs::write(&file_path, modified_toml) {
        println!("Unable to write file: {}", file_path);
        return 1;
    }

    return 0;

}

// dll exported function to return a pointer to a toml_edit::Doc, which can be used in other .dll functions
// takes a TOML string as an input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_from_string (
    toml_str: *const c_char,
) -> *mut c_void {
    let toml_str = unsafe { CStr::from_ptr(toml_str).to_string_lossy().into_owned() };

    let doc = match toml_edit::Document::from_str(&toml_str) {
        Ok(doc) => doc,
        Err(_) => {
            println!("Unable to parse TOML string: {}", toml_str);
            return ptr::null_mut();
        }
    };

    let doc = Box::new(doc);

    Box::into_raw(doc) as *mut c_void
}

// dll exported function to list the tables in a toml_edit::Doc as a multi-line string
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_list_tables (
    doc: *mut c_void,
) -> *mut c_char {
    let doc = unsafe { &mut *(doc as *mut toml_edit::Document) };

    let mut table_list = String::new();

    for table in doc.as_table() {
        table_list.push_str(&format!("{}\n", table.0));
    }

    let raw_string = match CString::new(table_list).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;
}


// dll exported function to return a pointer to a toml_edit::Table, which can be used in other .dll functions
// takes a toml_edit::Doc and a table name as inputs
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_get_table (
    doc: *mut c_void,
    table_name: *const c_char,
) -> *mut c_void {
    let doc = unsafe { &mut *(doc as *mut toml_edit::Document) };
    let table_name = unsafe { CStr::from_ptr(table_name).to_string_lossy().into_owned() };

    let table = match doc[table_name.as_str()].as_table() {
        Some(table) => table,
        None => {
            println!("Unable to find table: {}", table_name);
            return ptr::null_mut();
        }
    };

    let table = Box::new(table.clone());

    Box::into_raw(table) as *mut c_void
}

// dll exported function to return a pointer to a toml_edit::Table, which can be used in other .dll functions
// takes a toml_edit::Table and a sub-table name as inputs
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_get_sub_table (
    table: *mut c_void,
    sub_table_name: *const c_char,
) -> *mut c_void {
    let table = unsafe { &mut *(table as *mut toml_edit::Table) };
    let sub_table_name = unsafe { CStr::from_ptr(sub_table_name).to_string_lossy().into_owned() };

    let sub_table = match table[sub_table_name.as_str()].as_table() {
        Some(sub_table) => sub_table,
        None => {
            println!("Unable to find sub_table: {}", sub_table_name);
            return ptr::null_mut();
        }
    };

    let sub_table = Box::new(sub_table.clone());

    Box::into_raw(sub_table) as *mut c_void
}

// dll exported function to list sub-tables in a toml_edit::Table as a multi-line string
// takes a toml_edit::Table as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_list_sub_tables (
    table: *mut c_void,
) -> *mut c_char {
    let table = unsafe { &mut *(table as *mut toml_edit::Table) };

    let mut table_list = String::new();

    for item in table.iter() {
        if item.1.is_table() {
            table_list.push_str(&format!("{}\n", item.0));
        }
        // table_list.push_str(&format!("{}\n", item.0));
    }

    let raw_string = match CString::new(table_list).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;
}

// dll exported function to list keys in a toml_edit::Table as a multi-line string
// takes a toml_edit::Table as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_list_keys (
    table: *mut c_void,
) -> *mut c_char {
    let table = unsafe { &mut *(table as *mut toml_edit::Table) };

    let mut item_list = String::new();

    for item in table.iter() {
        if !item.1.is_table() {
            item_list.push_str(&format!("{}\n", item.0));
        }
    }

    let raw_string = match CString::new(item_list).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;
}

#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_list_items (
    table: *mut c_void,
) -> *mut c_char {
    let table = unsafe { &mut *(table as *mut toml_edit::Table) };

    let mut item_list = String::new();

    for item in table.iter() {
        item_list.push_str(&format!("{}\n", item.0));
    }

    let raw_string = match CString::new(item_list).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;
}

// dll exported function to return a pointer to a toml_edit::Item, which can be used in other .dll functions
// takes a toml_edit::Table and a item name as inputs
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_get_item (
    table: *mut c_void,
    item_name: *const c_char,
) -> *mut c_void {
    let table = unsafe { &mut *(table as *mut toml_edit::Table) };
    let item_name = unsafe { CStr::from_ptr(item_name).to_string_lossy().into_owned() };

    let item = match table[item_name.as_str()].clone() {
        item => item,
        // _ => {
        //     println!("Unable to find item: {}", item_name);
        //     return ptr::null_mut();
        // }
    };

    let item = Box::new(item);

    Box::into_raw(item) as *mut c_void
}

// dll exported function to get the type of a toml_edit::Item
// takes a toml_edit::Item as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_get_item_type (
    item: *mut c_void,
) -> *mut c_char {
    let item = unsafe { &mut *(item as *mut toml_edit::Item) };

    let item_type = match item {
        toml_edit::Item::None => "None",
        toml_edit::Item::Value(_) => "Value",
        toml_edit::Item::ArrayOfTables(_) => "ArrayOfTables",
        toml_edit::Item::Table(_) => "Table",
    };

    let raw_string = match CString::new(item_type).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;
}


// dll exported function to get the type of a toml_edit::Value
// takes a toml_edit::Value as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_get_value_type (
    value: *mut c_void,
) -> *mut c_char {
    let value = unsafe { &mut *(value as *mut toml_edit::Value) };

    let value_type = match value {
        toml_edit::Value::String(_) => "String",
        toml_edit::Value::Integer(_) => "Integer",
        toml_edit::Value::Float(_) => "Float",
        toml_edit::Value::Boolean(_) => "Boolean",
        toml_edit::Value::Datetime(_) => "Datetime",
        toml_edit::Value::Array(_) => "Array",
        toml_edit::Value::InlineTable(_) => "InlineTable",
    };

    let raw_string = match CString::new(value_type).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;
}

// dll exported function to get a toml_edit::Value from a toml_edit::Item
// takes a toml_edit::Item as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_item_into_value (
    item: *mut c_void,
) -> *mut c_void {
    let item = unsafe { &mut *(item as *mut toml_edit::Item) };

    let value = match item {
        toml_edit::Item::Value(value) => value,
        _ => {
            println!("Item is not a Value");
            return ptr::null_mut();
        }
    };

    let value = Box::new(value.clone());

    Box::into_raw(value) as *mut c_void
}


// dll exported function to get a toml_edit::Table from a toml_edit::Item
// takes a toml_edit::Item as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_item_into_table (
    item: *mut c_void,
) -> *mut c_void {
    let item = unsafe { &mut *(item as *mut toml_edit::Item) };

    let table = match item {
        toml_edit::Item::Table(table) => table,
        _ => {
            println!("Item is not a Table");
            return ptr::null_mut();
        }
    };

    let table = Box::new(table.clone());

    Box::into_raw(table) as *mut c_void
}


// dll exported function to get a String typed Value from a toml_edit::Value
// takes a toml_edit::Value as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_get_value_string (
    value: *mut c_void,
) -> *mut c_char {
    let value = unsafe { &mut *(value as *mut toml_edit::Value) };

    let value = match value {
        toml_edit::Value::String(value) => value,
        _ => {
            println!("Value is not a String");
            return CString::new("").unwrap().into_raw();
        }
    };

    let return_value = value.clone().into_value();

    let raw_string = match CString::new(return_value).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;
}

// dll exported function to get a i64 typed Value from a toml_edit::Value
// takes a toml_edit::Value as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_get_value_i64 (
    value: *mut c_void,
) -> i64 {
    let value = unsafe { &mut *(value as *mut toml_edit::Value) };

    let value = match value {
        toml_edit::Value::Integer(value) => value,
        _ => {
            println!("Value is not a Integer");
            return 0;
        }
    };

    let return_value = value.clone().into_value();

    return return_value;
}

// dll exported function to get an InlineTable typed Value from a toml_edit::Value
// takes a toml_edit::Value as input and returns a raw pointer to a toml_edit::Table
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_get_value_inline_table (
    value: *mut c_void,
) -> *mut c_void {
    let value = unsafe { &mut *(value as *mut toml_edit::Value) };

    let value = match value {
        toml_edit::Value::InlineTable(value) => value,
        _ => {
            println!("Value is not a InlineTable");
            return ptr::null_mut();
        }
    };

    let value = Box::new(value.clone());

    Box::into_raw(value) as *mut c_void
}

// dll exported to return a multi-line string of the keynames in an InlineTable
// takes a toml_edit::InlineTable as input
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_inline_table_keynames (
    table: *mut c_void,
) -> *mut c_char {
    let table = unsafe { &mut *(table as *mut toml_edit::InlineTable) };

    let mut return_string = String::new();

    for (key, _) in table.iter() {
        return_string.push_str(&key);
        return_string.push_str("\n");
    }

    let raw_string = match CString::new(return_string).unwrap().into_raw() {
        ptr if ptr.is_null() => {
            println!("Unable to allocate memory for string");
            return CString::new("").unwrap().into_raw();
        },
        ptr => ptr,
    };

    return raw_string;
}


// dll exported function to close the toml_edit::Doc and free the memory
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_close (
    doc: *mut c_void,
) {
    let doc = unsafe { Box::from_raw(doc as *mut toml_edit::Document) };
    drop(doc);
}

// dll exported function to close the toml_edit::Table and free the memory
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_close_table (
    table: *mut c_void,
) {
    let table = unsafe { Box::from_raw(table as *mut toml_edit::Table) };
    drop(table);
}

// dll exported function to close the toml_edit::Item and free the memory
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_close_item (
    item: *mut c_void,
) {
    let item = unsafe { Box::from_raw(item as *mut toml_edit::Item) };
    drop(item);
}

// dll exported function to close the toml_edit::Value and free the memory
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_close_value (
    value: *mut c_void,
) {
    let value = unsafe { Box::from_raw(value as *mut toml_edit::Value) };
    drop(value);
}

// dll exported function to close the toml_edit::InlineTable and free the memory
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn toml_edit_close_inline_table (
    table: *mut c_void,
) {
    let table = unsafe { Box::from_raw(table as *mut toml_edit::InlineTable) };
    drop(table);
}


// a DLL function that frees the memory allocated for a string
#[no_mangle]
pub extern "C" fn memory_free_string(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}


// a DLL function that returns a list of packages (as a multi-line string)
#[allow(dead_code)]
#[no_mangle]
pub extern "C" fn list_packages (
    file_path: *const c_char,
    package_manager: *const c_char,
) -> *mut c_char {
    let file_path = unsafe { CStr::from_ptr(file_path).to_string_lossy().into_owned() };
    let package_manager = unsafe { CStr::from_ptr(package_manager).to_string_lossy().into_owned() };

    let package_manager = match parse_package_manager(&package_manager) {
        Ok(manager) => manager,
        Err(_) => {
            println!("Invalid package_manager argument:
            {}", package_manager);
            return CString::new("").unwrap().into_raw();
        }
    };

    let toml_str = match fs::read_to_string(&file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Unable to read file: {}", file_path);
            return CString::new("").unwrap().into_raw();
        }
    };

    let package_list = match toml_list_packages(
        toml_str,
        package_manager,
    ) {
        Ok(list) => list,
        Err(e) => {
            println!("Error modifying toml file: {}", e);
            return CString::new("").unwrap().into_raw();
        }
    };

    return CString::new(package_list).unwrap().into_raw();
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    // function to remove leading whitespace from each line in a string.
    fn remove_indentation(s: &str) -> String {
        let mut result = String::new();
        for line in s.lines() {
            let trimmed_line = line.trim_start();
            result.push_str(trimmed_line);
            result.push_str("\n");
        }

        // remove leading and trailing newlines
        result = result.trim().to_string();

        return result;
    }

    // function to assert that two strings are equal, ignoring indentation and leading/trailing newlines.
    fn assert_equal_ignore_indentation(s1: &str, s2: &str) {
        let s1 = remove_indentation(s1);
        let s2 = remove_indentation(s2);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_modify_toml() {
        let toml_str = "";
        let modified_toml = toml_set_package_attribute_string(
            toml_str.to_string(),
            PackageManager::VIPM,
            "vipm_package",
            "version",
            "1.2.3",
        );
        assert_equal_ignore_indentation(
            modified_toml.unwrap().as_str(),
            r#"
            [vipm]

            [vipm.dependencies]
            vipm_package = "1.2.3"
            "#
        );
    }



    #[test]
    fn test_replace_version_subkey() {
        let toml_str = r#"
            [nipm]

            [nipm.dependencies]
            my_package = { version = "0.0.0", url = "https://package.net/my_package/0.0.0" }
            "#;

        let modified_toml = toml_set_package_attribute_string(
            toml_str.to_string(),
            PackageManager::NIPM,
            "my_package",
            "version",
            "1.2.3",
        );

        let expected_toml = r#"
            [nipm]

            [nipm.dependencies]
            my_package = { version = "1.2.3", url = "https://package.net/my_package/0.0.0" }
            "#;

        assert_equal_ignore_indentation(modified_toml.unwrap().as_str(), expected_toml);
    }

    #[test]
    fn test_replace_version_string() {
        let toml_str = r#"
            [nipm]

            [nipm.dependencies]
            my_package = "1.0.0"
            "#;

        let modified_toml = toml_set_package_attribute_string(
            toml_str.to_string(),
            PackageManager::NIPM,
            "my_package",
            "version",
            "1.2.3",
        );

        let expected_toml = r#"
            [nipm]

            [nipm.dependencies]
            my_package = "1.2.3"
            "#;

        assert_equal_ignore_indentation(modified_toml.unwrap().as_str(), expected_toml);
    }


    #[test]
    fn test_add_to_inline_table() {
        let toml_str = r#"
            [vipm]

            [vipm.dependencies]
            my_package = "1.0.0"
            "#;

        let modified_toml = toml_set_package_attribute_string(
            toml_str.to_string(),
            PackageManager::VIPM,
            "my_package",
            "url",
            "https://jki.net",
        );
        assert_equal_ignore_indentation(
            modified_toml.unwrap().as_str(),
            r#"
            [vipm]

            [vipm.dependencies]
            my_package = { version = "1.0.0", url = "https://jki.net" }
            "#
        );
    }

    #[test]
    fn test_toml_set_package_version() {
        let toml_str = r#"
            [nipm]

            [nipm.dependencies]
            my_package = { version = "1.0.0", url = "https://jki.net" }
            "#;

        let modified_toml = toml_set_package_attribute_string(
            toml_str.to_string(),
            PackageManager::NIPM,
            "my_package",
            "version",
            "1.2.3",
        );
        assert_equal_ignore_indentation(
            modified_toml.unwrap().as_str(),
            r#"
            [nipm]

            [nipm.dependencies]
            my_package = { version = "1.2.3", url = "https://jki.net" }
            "#
        );
    }

    #[test]
    fn test_preserve_comment() {
        let toml_str = r#"
            [nipm]

            [nipm.dependencies]
            # this is a comment
            my_package = "1.0.0"
            "#;

        let modified_toml = toml_set_package_attribute_string(
            toml_str.to_string(),
            PackageManager::NIPM,
            "my_package",
            "version",
            "1.2.3",
        );
        assert_equal_ignore_indentation(
            modified_toml.unwrap().as_str(),
            r#"
            [nipm]

            [nipm.dependencies]
            # this is a comment
            my_package = "1.2.3"
            "#
        );
    }

    #[test]
    fn test_create_tables_if_needed() {
        let toml_str = r#"
            "#;

        let modified_toml = toml_set_package_attribute_string(
            toml_str.to_string(),
            PackageManager::NIPM,
            "my_package",
            "version",
            "1.2.3",
        );
        assert_equal_ignore_indentation(
            modified_toml.unwrap().as_str(),
            r#"
            [nipm]

            [nipm.dependencies]
            my_package = "1.2.3"
            "#
        );
    }

    #[test]
    fn test_toml_get_package_version() {
        let toml_str = r#"
            [nipm]

            [nipm.dependencies]
            my_package = { version = "1.0.0", url = "https://jki.net" }
            "#;

        let version_string = toml_get_package_attribute_string(
            toml_str.to_string(),
            PackageManager::NIPM,
            "my_package",
            "version",
        );
        assert_equal_ignore_indentation(
            version_string.unwrap().as_str(),
            r#"1.0.0"#
        );
    }

    #[test]
    fn test_toml_get_package_missing_attribute() {
        let toml_str = r#"
            [nipm]

            [nipm.dependencies]
            my_package = { version = "1.0.0", url = "https://jki.net" }
            "#;

        let version_string = toml_get_package_attribute_string(
            toml_str.to_string(),
            PackageManager::NIPM,
            "my_package",
            "display_version",
        );

        // check if version_string contains an error message
        assert!(version_string.is_err());
    }

}