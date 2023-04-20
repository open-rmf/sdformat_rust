use std::env;
use std::fmt::format;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::error::Error;

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

fn return_type(typename: &str) -> String
{
    if typename == "string"
    {
        return "String".to_string();
    }
    else if typename == "double"
    {
        return "f64".to_string();
    }
    else if typename == "vector3"
    {
        return "Vector3<f64>".to_string();
    }
    else if typename == "vector2d"
    {
        return "Vector2<f64>".to_string();
    }
    else if typename == "vector2i"
    {
        return "Vector2<i64>".to_string();
    }
    else if typename == "pose"
    {
        return "Pose".to_string();
    }
    return typename.to_string();
}


struct SDFIncludes
{
    filename: String,
    required: String
}

struct SDFAttribute
{
    name: String,
    rtype: String,
    required: RequiredStatus,
    default: Option<String>
}

impl SDFAttribute
{
    fn new()-> Self {
        Self {
            name: "".to_string(),
            rtype: "".to_string(),
            required: RequiredStatus::Optional,
            default: None
        }
    }
    fn get_field_string(&self) -> String {

        format!("  #[yaserde(attribute, rename = \"{}\")]\n  _{}: {},\n",
            self.name,
            self.name,
            self.required.wrap_type(get_storage_type(self.rtype.as_str())))

    }

    fn getter_body(&self) -> String
    {
        if return_type(self.rtype.as_str()) == self.rtype {
            format!("  self._{}", self.name)
        }
        else {
            format!("")
        }
    }


    fn getter(&self) -> String {
        format!(r#"pub fn get_{}(&self) -> {} {{
            {}
        }}"#, self.name, return_type(self.rtype.as_str()), self.getter_body())
    }
}

struct SDFElement
{
    properties: SDFAttribute,
    child_elems: Vec<SDFElement>,
    child_attrs: Vec<SDFAttribute>,
    child_includes: Vec<SDFIncludes>,
    source_file: String
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
                default: None
            },
            child_elems: vec!(),
            child_attrs: vec!(),
            child_includes: vec!(),
            source_file: "".to_string()
        }
    }



    fn code_gen(&self, prefix: &str) -> String
    {

        let mut out = "".to_string();
        out += "#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]\n";
        out += format!("#[yaserde(rename = \"{}\")]\n", self.properties.name).as_str();
        out += format!("pub struct {}{} {{\n", prefix, self.properties.name).as_str();
        for child in &self.child_attrs {
            out += child.get_field_string().as_str();
        }

        let mut child_gen = "".to_string();
        let name = self.properties.name.as_str();
        for child in &self.child_elems {
            if child.properties.rtype == "" {
                // TODO(arjo): Handle includes
                let prefix = name.to_owned() +"_";
                child_gen += child.code_gen(prefix.as_str()).as_str();
                let typename = prefix + child.properties.name.as_str();
                out +=
                    format!(
                        "  #[yaserde(child, rename = \"{}\")]\n  _{}: {},\n",
                        child.properties.name,
                        child.properties.name,
                        child.properties.required.wrap_type(typename.as_str())
                    ).as_str();

            }
            else
            {
                let typename = get_storage_type(child.properties.rtype.as_str());
                out +=
                    format!(
                        "  #[yaserde(child, rename = \"{}\")]\n  _{}: {},\n",
                        child.properties.name,
                        child.properties.name,
                        child.properties.required.wrap_type(typename)
                    ).as_str();
            }
        }
        out += "}\n\n";
        out += child_gen.as_str();

        out += format!("impl {}{} {{\n", prefix, self.properties.name).as_str();
        for child in &self.child_attrs {
           // TODO(arjo): work out getter
           // out += child.getter().as_str();
        }
        out += "}\n";
        out
    }
}

fn parse_element(model: &mut SDFElement, element: &Element) {

    if element.name == "element"
    {
        //let mut child_elem = SDFElement::new();
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
            required: element.attributes.get("required").unwrap().to_string()
        };
        model.child_includes.push(incl);
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
                    let mut elem = SDFElement::new();
                    parse_element(&mut elem, &el);
                    model.child_elems.push(elem);
                }
            }
            _ => {

            }
        }
    }
}


fn main() {

    let spec = read_spec("sdformat_spec/1.10/camera.sdf").unwrap();

    let mut model = SDFElement::new();
    parse_element(&mut model, &spec);
    let contents = model.code_gen("");

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
