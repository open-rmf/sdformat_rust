use std::collections::{hash_map, HashSet};
use std::{env, collections::HashMap};
use std::fmt::format;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::error::Error;
use convert_case::{Case, Casing};
use minidom::element;
use xmltree::{Element, XMLNode};

fn read_spec(filename: &str) -> Result<Element, Box<dyn Error>>
{
    let addr = Element::parse(
        Cursor::new(
            fs::read(filename)?)).unwrap();
    Ok(addr)
}


fn get_storage_type<'a>(type_str: &str) -> &'a str
{
    if type_str == "double" {
        return "f64";
    }

    // TODO(arjo): SDF bool is a bit funny and probably needs type support
    if type_str == "bool" {
        return "bool";
    }

    if type_str == "vector3" {
        return "Vector3d";
    }
    return "String";
}

enum RequiredStatus {
    Optional,
    One,
    Many
}

impl RequiredStatus {
    fn wrap_type(&self, type_str: &str) -> String {
        match self {
            RequiredStatus::Optional => {
                format!("Option<{}>", type_str)
            },
            RequiredStatus::One => {
                format!("{}", type_str)
            },
            RequiredStatus::Many => {
                format!("Vec<{}>", type_str)
            }
        }
    }

    fn from_str(required: &str) -> RequiredStatus {
        if required == "true" || required == "1" {
            return RequiredStatus::One;
        }
        else if required == "*" {
            return RequiredStatus::Many;
        }
        RequiredStatus::Optional
    }
}

fn sanitize_field(fieldname: &str) -> String {
    let hashset = HashSet::from(["loop", "static", "type", "box"]);

    if hashset.contains(fieldname) {
        format!("r#{}", fieldname)
    }
    else {
        fieldname.to_string()
    }
}

struct SDFIncludes
{
    filename: String,
    required: RequiredStatus
}

struct SDFAttribute
{
    name: String,
    rtype: String,
    required: RequiredStatus,
    default: Option<String>,
    description: String
}

impl SDFAttribute
{
    fn new()-> Self {
        Self {
            name: "".to_string(),
            rtype: "".to_string(),
            required: RequiredStatus::Optional,
            default: None,
            description: "".to_string()
        }
    }
    fn get_field_string(&self) -> String {
        format!("  #[yaserde(attribute, rename = \"{}\")]\n  pub {}: {},\n",
            self.name,
            sanitize_field(&self.name),
            self.required.wrap_type(get_storage_type(self.rtype.as_str())))
    }
}

fn prefix_type(name: &str) -> String
{
    if name.starts_with("Sdf") {
        name.to_case(Case::Pascal)
    }
    else
    {
        "Sdf".to_string() + name.to_case(Case::Pascal).as_str()
    }
}

struct SDFElement
{
    properties: SDFAttribute,
    child_elems: Vec<SDFElement>,
    child_attrs: Vec<SDFAttribute>,
    child_includes: Vec<SDFIncludes>,
    source_file: String,
    top_level: bool
}

impl SDFElement
{
    fn new() -> Self
    {
        Self{
            properties: SDFAttribute {
                name: "".to_string(),
                rtype: "".to_string(),
                required: RequiredStatus::Optional,
                default: None,
                description: "".to_string(),
            },
            child_elems: vec!(),
            child_attrs: vec!(),
            child_includes: vec!(),
            source_file: "".to_string(),
            top_level: false
        }
    }

    fn typename(&self) -> String {
        if self.top_level {
            self.source_file[..self.source_file.len()-4].to_string().to_case(Case::Pascal)
        }
        else {
            self.properties.name.to_case(Case::Pascal)
        }
    }


    fn code_gen(&self, prefix: &str, file_map: &HashMap<String, SDFElement>) -> String
    {

        let mut out = "".to_string();
        out += format!("/// Generated from {}\n", self.source_file).as_str();
        out += "#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]\n";
        out += format!("#[yaserde(rename = \"{}\")]\n", self.properties.name).as_str();
        if self.top_level {
            out += format!("pub struct {}{} {{\n", prefix_type(prefix), self.typename()).as_str();
        }
        else {
            out += format!("pub struct {}{} {{\n", prefix_type(prefix), self.properties.name.to_case(Case::Pascal)).as_str();
        }
        for child in &self.child_attrs {
            out += child.get_field_string().as_str();
        }

        let mut child_gen = "".to_string();
        let name = prefix.to_string().to_case(Case::Pascal) + self.properties.name.as_str();
        for child in &self.child_elems {
            if child.properties.rtype == "" {
                // TODO(arjo): Handle includes
                let prefix = prefix_type(&name);
                child_gen += child.code_gen(prefix.as_str(), file_map).as_str();
                let typename = prefix + child.properties.name.to_case(Case::Pascal).as_str();
                out +=
                    format!(
                        "  #[yaserde(child, rename = \"{}\")]\n  pub {}: {},\n",
                        child.properties.name,
                        &sanitize_field(&child.properties.name),
                        child.properties.required.wrap_type(typename.to_case(Case::Pascal).as_str())
                    ).as_str();

            }
            else
            {
                let typename = get_storage_type(child.properties.rtype.as_str());
                out +=
                    format!(
                        "  #[yaserde(child, rename = \"{}\")]\n  pub {}: {},\n",
                        child.properties.name,
                        &sanitize_field(&child.properties.name),
                        child.properties.required.wrap_type(typename)
                    ).as_str();
            }
        }
        for child in &self.child_includes {
            if let Some(element) = file_map.get(&child.filename.to_string()) {
                
                let typename = child.required.wrap_type(
                    &("Sdf".to_string() + element.typename().as_str()));
                out += format!("  #[yaserde(child, rename = \"{}\")]\n  pub {} : {},\n",
                    element.properties.name.to_case(Case::Snake),
                    &sanitize_field(&element.properties.name.to_case(Case::Snake)), 
                    typename).as_str();
            }
            else {
                panic!("Unable to find element for file: {}", child.filename);
            }
        }
        if self.properties.rtype.len() > 0 {
            out += format!("#[yaserde(text)]\n   data: String\n").as_str();//format!("data: {}", self.properties.rtype).as_str(); 
        }
        out += "}\n\n";
        out += child_gen.as_str();
        out
    }

    fn set_source(&mut self, filename: &str){
        for elem in &mut self.child_elems {
            elem.set_source(filename);
        }
        self.source_file = filename.to_string();
    }
}

fn parse_element(model: &mut SDFElement, element: &Element) {

    if element.name == "element"
    {
        // Parse element description
        if let Some(name)= element.attributes.get("name")
        {
            model.properties.name = name.clone();
        }
        if let Some(rtype)= element.attributes.get("type")
        {
            model.properties.rtype = rtype.clone();
        }
        if let Some(default)= element.attributes.get("default")
        {
            model.properties.default = Some(default.clone());
        }
        if let Some(required)= element.attributes.get("required")
        {
            model.properties.required = RequiredStatus::from_str(required);
        }
    }
    else if element.name == "attribute"
    {
        let mut attr = SDFAttribute::new();
        // Parse element description
        if let Some(name) = element.attributes.get("name")
        {
            attr.name = name.clone();
        }
        if let Some(rtype) = element.attributes.get("type")
        {
            attr.rtype = rtype.clone();
        }
        if let Some(default) = element.attributes.get("default")
        {
            attr.default = Some(default.clone());
        }
        if let Some(required)= element.attributes.get("required")
        {
            attr.required = RequiredStatus::from_str(required);
        }
        model.child_attrs.push(attr);
    }
    else if element.name == "include"
    {
        let incl = SDFIncludes {
            filename: element.attributes.get("filename").unwrap().to_string(),
            required: RequiredStatus::from_str(element.attributes.get("required").unwrap())
        };
        model.child_includes.push(incl);
    }
    else if element.name == "description"
    {
        model.properties.description = element.get_text().unwrap().to_string();
    }

    for child in &element.children {
        match child {
            XMLNode::Element(el) => {
                if el.name == "attribute"
                {
                    parse_element(model, &el);
                }
                else if el.name == "element"
                {
                    if el.attributes.contains_key("ref")
                    {

                    }
                    else {
                        let mut elem = SDFElement::new();
                        parse_element(&mut elem, &el);
                        model.child_elems.push(elem);
                    }
                }
                else if el.name == "include"
                {
                    let incl = SDFIncludes {
                        filename: el.attributes.get("filename").unwrap().to_string(),
                        required: RequiredStatus::from_str(element.attributes.get("required").unwrap())
                    };
                    model.child_includes.push(incl);
                }
                else if el.name == "description"
                {
                    if let Some(desc) = el.get_text()
                    {
                        model.properties.description = desc.to_string();
                    }
                }

            }
            _ => {

            }
        }
    }
}

fn read_all_specs() -> Result<HashMap<String, SDFElement>, String> {
    let mut res = HashMap::new();
    for file in std::fs::read_dir("sdformat_spec/1.10").unwrap() {
        if let Ok(dir_entry) = file {
            if !dir_entry.metadata().unwrap().is_file() {
                continue;
            }
            if let Some(sdf) = dir_entry.path().extension() {
                if sdf == "sdf" {
                    let spec = read_spec(dir_entry.path().to_str().unwrap()).unwrap();
                    let mut model = SDFElement::new();
                    parse_element(&mut model, &spec);
                    model.top_level = true;
                    model.set_source(dir_entry.file_name().to_str().unwrap());
                    res.insert(dir_entry.file_name().to_str().unwrap().to_string(), model);
                }
            }
        }
    }
    

    Ok(res)
}

fn main() {

    let hashmap = read_all_specs().unwrap();

    let mut contents = "".to_string();
    for (file, model) in &hashmap {
        if file == "plugin.sdf" || file == "frame.sdf" {
            //Skip
            continue
        }
        contents += &model.code_gen("", &hashmap);
    }

    // For debug
    fs::write("test_codegen.rs", contents.clone());

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("sdf.rs");
    fs::write(
        &dest_path,
        contents
    ).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
