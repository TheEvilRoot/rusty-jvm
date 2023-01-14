use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Error, Read, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::loader::ClassLoadError::UnknownElementValueTag;
use crate::loader::ElementValue::{AnnotationValue, ArrayValue, ClassInfoIndex, ConstValueIndex, EnumConstValue};

#[derive(Debug)]
pub enum ClassLoadError {
    MagicMismatch(u32),
    VersionUnsupported(u16, u16),
    ClassFileReadFailure(Error),
    ConstantPoolMissing(u16),
    AttributeMissing(String),
    AttributeTypeMismatch(String, String),
    ConstantPoolTypeMismatch(String, String),
    UnknownElementValueTag(u8)
}

impl From<Error> for ClassLoadError {
    fn from(e: Error) -> Self {
        ClassLoadError::ClassFileReadFailure(e)
    }
}

trait AttributesContainer {
    fn get_attributes(&self) -> &AttributeInfo;
}

impl dyn AttributesContainer {
    
}

#[derive(Debug)]
enum ConstantPoolTag {
    Class(u16),
    FieldRef(u16, u16),
    MethodRef(u16, u16),
    InterfaceMethodRef(u16, u16),
    String(u16),
    Integer(u32),
    Float(u32),
    Long(u32, u32),
    Double(u32, u32),
    NameAndType(u16, u16),
    Utf8(u16, Vec<u8>, String),
    MethodHandle(u8, u16),
    MethodType(u16),
    InvokeDynamic(u16, u16),
    Dummy
}

impl ConstantPoolTag {
    fn from_reader(reader: &mut File) -> Result<Vec<ConstantPoolTag>, ClassLoadError> {
        let byte = reader.read_u8()?;
        return Ok(match byte {
            1 => {
                let length = reader.read_u16::<BigEndian>()?;
                let mut bytes = vec![0u8; length as usize];
                reader.read_exact(&mut bytes)?;
                let string = String::from_utf8(bytes.clone()).unwrap_or("<nil>".to_string());
                vec![ConstantPoolTag::Utf8(length, bytes, string)]
            },
            3 => vec![ConstantPoolTag::Integer(reader.read_u32::<BigEndian>()?)],
            4 => vec![ConstantPoolTag::Float(reader.read_u32::<BigEndian>()?)],
            5 => vec![ConstantPoolTag::Long(reader.read_u32::<BigEndian>()?, reader.read_u32::<BigEndian>()?), ConstantPoolTag::Dummy],
            6 => vec![ConstantPoolTag::Double(reader.read_u32::<BigEndian>()?, reader.read_u32::<BigEndian>()?), ConstantPoolTag::Dummy],
            7 => vec![ConstantPoolTag::Class(reader.read_u16::<BigEndian>()?)],
            8 => vec![ConstantPoolTag::String(reader.read_u16::<BigEndian>()?)],
            9 => vec![ConstantPoolTag::FieldRef(reader.read_u16::<BigEndian>()?, reader.read_u16::<BigEndian>()?)],
            10 => vec![ConstantPoolTag::MethodRef(reader.read_u16::<BigEndian>()?, reader.read_u16::<BigEndian>()?)],
            11 => vec![ConstantPoolTag::InterfaceMethodRef(reader.read_u16::<BigEndian>()?, reader.read_u16::<BigEndian>()?)],
            12 => vec![ConstantPoolTag::NameAndType(reader.read_u16::<BigEndian>()?, reader.read_u16::<BigEndian>()?)],
            15 => vec![ConstantPoolTag::MethodHandle(reader.read_u8()?, reader.read_u16::<BigEndian>()?)],
            16 => vec![ConstantPoolTag::MethodType(reader.read_u16::<BigEndian>()?)],
            18 => vec![ConstantPoolTag::InvokeDynamic(reader.read_u16::<BigEndian>()?, reader.read_u16::<BigEndian>()?)],
            _ => panic!("Unexpected constant pool tag: {} at 0x{:x}", byte, reader.seek(SeekFrom::Current(0))?)
        });
    }
}

#[derive(Debug)]
struct ExceptionEntry {
    pc_start: u16,
    pc_end: u16,
    handler_pc: u16,
    catch_type: u16
}

impl ExceptionEntry {
    fn from_reader(reader: &mut Cursor<&&Vec<u8>>) -> Result<ExceptionEntry, ClassLoadError> {
        let pc_start = reader.read_u16::<BigEndian>()?;
        let pc_end = reader.read_u16::<BigEndian>()?;
        let handler_pc = reader.read_u16::<BigEndian>()?;
        let catch_type = reader.read_u16::<BigEndian>()?;
        Ok(ExceptionEntry {
            pc_start,
            pc_end,
            handler_pc,
            catch_type
        })
    }
}

#[derive(Debug)]
struct ElementValuePair {
    name_index: u16,
    value: ElementValue
}

impl ElementValuePair {
    fn from_cursor(cursor: &mut Cursor<&&Vec<u8>>) -> Result<Self, ClassLoadError> {
        let name_index = cursor.read_u16::<BigEndian>()?;
        let value = ElementValue::from_cursor(cursor)?;
        Ok(ElementValuePair {
            name_index,
            value
        })
    }
}

#[derive(Debug)]
struct Annotation {
    type_index: u16,
    num_element_value_pairs: u16,
    element_value_pairs: Vec<ElementValuePair>
}

impl Annotation {
    fn from_cursor(cursor: &mut Cursor<&&Vec<u8>>) -> Result<Self, ClassLoadError> {
        let type_index = cursor.read_u16::<BigEndian>()?;
        let num_element_value_pairs = cursor.read_u16::<BigEndian>()?;
        let mut pairs: Vec<ElementValuePair> = Vec::new();
        for _ in 0 .. num_element_value_pairs {
            pairs.push(ElementValuePair::from_cursor(cursor)?);
        }
        Ok(Annotation {
            type_index,
            num_element_value_pairs,
            element_value_pairs: pairs
        })
    }
}

#[derive(Debug)]
enum ElementValue {
    ConstValueIndex(u16),
    EnumConstValue(u16, u16),
    ClassInfoIndex(u16),
    AnnotationValue(Annotation),
    ArrayValue(u16, Vec<ElementValue>)
}

impl ElementValue {
    fn from_cursor(cursor: &mut Cursor<&&Vec<u8>>) -> Result<Self, ClassLoadError> {
        let tag = cursor.read_u8()?;
        match tag as char {
            's'
            | 'B'
            | 'C'
            | 'D'
            | 'F'
            | 'I'
            | 'J'
            | 'S'
            | 'Z' => Ok(ConstValueIndex(cursor.read_u16::<BigEndian>()?)),
            'e' => Ok(EnumConstValue(cursor.read_u16::<BigEndian>()?, cursor.read_u16::<BigEndian>()?)),
            'c' => Ok(ClassInfoIndex(cursor.read_u16::<BigEndian>()?)),
            '@' => Ok(AnnotationValue(Annotation::from_cursor(cursor)?)),
            '[' => {
                let num_values = cursor.read_u16::<BigEndian>()?;
                let mut values: Vec<ElementValue> = Vec::new();
                for _ in 0 .. num_values {
                    values.push(ElementValue::from_cursor(cursor)?);
                }
                Ok(ArrayValue(num_values, values))
            }
            _ => Err(UnknownElementValueTag(tag))
        }
    }
}

#[derive(Debug)]
struct ParameterAnnotation {
    num_annotations: u16,
    annotations: Vec<Annotation>
}

impl ParameterAnnotation {
    fn from_cursor(cursor: &mut Cursor<&&Vec<u8>>) -> Result<Self, ClassLoadError> {
        let num_annotations = cursor.read_u16::<BigEndian>()?;
        let mut annotations: Vec<Annotation> = Vec::new();
        for _ in 0 .. num_annotations {
            annotations.push(Annotation::from_cursor(cursor)?);
        }
        Ok(ParameterAnnotation {
            num_annotations,
            annotations
        })
    }
}

#[derive(Debug)]
enum AttributeValue {
    ConstantValue(u16),
    SourceFile(u16),
    Code(u16, u16, u32, Vec<u8>, u16, Vec<ExceptionEntry>, u16, Vec<AttributeInfo>),
    RuntimeInvisibleParameterAnnotations(u8, Vec<ParameterAnnotation>),
    RuntimeInvisibleAnnotations(u16, Vec<Annotation>),
    Unidentified(Vec<u8>),
}

impl AttributeValue {
    fn from_name_and_info(name: &str, info: &Vec<u8>) -> Result<Self, ClassLoadError> {
        let mut cursor = Cursor::new(&info);
        Ok(match name {
            "ConstantValue" => AttributeValue::ConstantValue(cursor.read_u16::<BigEndian>()?),
            "SourceFile" => AttributeValue::SourceFile(cursor.read_u16::<BigEndian>()?),
            "Code" => {
                let max_stack = cursor.read_u16::<BigEndian>()?;
                let max_locals = cursor.read_u16::<BigEndian>()?;
                let code_length = cursor.read_u32::<BigEndian>()?;
                let mut code = vec![0u8; code_length as usize];
                cursor.read_exact(&mut code)?;
                let exc_table_length = cursor.read_u16::<BigEndian>()?;
                let mut exc_table: Vec<ExceptionEntry> = Vec::new();
                for _ in 0 .. exc_table_length {
                    exc_table.push(ExceptionEntry::from_reader(&mut cursor)?);
                }
                let attributes_count = cursor.read_u16::<BigEndian>()?;
                let mut attr_table: Vec<AttributeInfo> = Vec::new();
                for _ in 0 .. attributes_count {
                    attr_table.push(AttributeInfo::from_cursor(&mut cursor)?);
                }
                AttributeValue::Code(
                    max_stack,
                    max_locals,
                    code_length,
                    code,
                    exc_table_length,
                    exc_table,
                    attributes_count,
                    attr_table
                )
            }
            "RuntimeInvisibleParameterAnnotations" => {
                let num_parameters = cursor.read_u8()?;
                let mut parameters: Vec<ParameterAnnotation> = Vec::new();
                for _ in 0 .. num_parameters {
                    parameters.push(ParameterAnnotation::from_cursor(&mut cursor)?);
                }
                AttributeValue::RuntimeInvisibleParameterAnnotations(num_parameters, parameters)
            }
            "RuntimeInvisibleAnnotations" => {
                let num_annotations = cursor.read_u16::<BigEndian>()?;
                let mut annotations: Vec<Annotation> = Vec::new();
                for _ in 0 .. num_annotations {
                    annotations.push(Annotation::from_cursor(&mut cursor)?);
                }
                AttributeValue::RuntimeInvisibleAnnotations(num_annotations, annotations)
            }
            _ => AttributeValue::Unidentified(info.clone())
        })

    }
}

struct ClassFileConstantPool {
    constant_pool_count: u16,
    constant_pool: Vec<ConstantPoolTag>
}

impl ClassFileConstantPool {
    fn from_reader(reader: &mut File) -> Result<ClassFileConstantPool, ClassLoadError> {
        let constant_pool_count = reader.read_u16::<BigEndian>()?;
        let mut constant_pool: Vec<ConstantPoolTag> = vec![];
        let mut entry_index: usize = 1;
        println!("from_reader constant_pool size {}", constant_pool_count);
        loop {
            if constant_pool.len() >= (constant_pool_count - 1) as usize {
                break
            }
            let tags = ConstantPoolTag::from_reader(&mut *reader)?;
            println!("from_reader constant_pool {} = {:?}", entry_index, tags);
            entry_index += tags.len();
            constant_pool.extend(tags);
        }
        Ok(ClassFileConstantPool {
            constant_pool_count,
            constant_pool
        })
    }
}

#[derive(Debug)]
enum AccessFlags {
    Public,
    Final,
    Super,
    Interface,
    Abstract,
    Synthetic,
    Annotation,
    Enum,
    Private,
    Protected,
    Static,
    Volatile,
    Transient,
    Synchronized,
    Bridge,
    Varargs,
    Native,
    Strict
}

impl AccessFlags {
    fn from_reader(reader: &mut File, is_method: bool) -> Result<Vec<AccessFlags>, ClassLoadError> {
        let value = reader.read_u16::<BigEndian>()?;
        let mut ret: Vec<AccessFlags> = vec![];
        if value & 0x0001 == 0x0001 {
            ret.push(AccessFlags::Public)
        }
        if value & 0x0002 == 0x0002 {
            ret.push(AccessFlags::Private)
        }
        if value & 0x0004 == 0x0004 {
            ret.push(AccessFlags::Protected)
        }
        if value & 0x0008 == 0x0008 {
            ret.push(AccessFlags::Static)
        }
        if value & 0x0010 == 0x0010 {
            ret.push(AccessFlags::Final)
        }
        if value & 0x0020 == 0x0020 {
            if is_method {
                ret.push(AccessFlags::Synchronized)
            } else {
                ret.push(AccessFlags::Super)
            }
        }
        if value & 0x0040 == 0x0040 {
            if is_method {
                ret.push(AccessFlags::Bridge)
            } else {
                ret.push(AccessFlags::Volatile)
            }
        }
        if value & 0x0080 == 0x0080 {
            if is_method {
                ret.push(AccessFlags::Varargs)
            } else {
                ret.push(AccessFlags::Transient)
            }
        }
        if value & 0x0100 == 0x0100 {
            ret.push(AccessFlags::Native)
        }
        if value & 0x0200 == 0x0200 {
            ret.push(AccessFlags::Interface)
        }
        if value & 0x0400 == 0x0400 {
            ret.push(AccessFlags::Abstract)
        }
        if value & 0x0800 == 0x0800 {
            ret.push(AccessFlags::Strict)
        }
        if value & 0x1000 == 0x1000 {
            ret.push(AccessFlags::Synthetic)
        }
        if value & 0x2000 == 0x2000 {
            ret.push(AccessFlags::Annotation)
        }
        if value & 0x4000 == 0x4000 {
            ret.push(AccessFlags::Enum)
        }
        Ok(ret)
    }
}

struct ClassFileInterfaces {
    interfaces_count: u16,
    interfaces: Vec<u16>
}

impl ClassFileInterfaces {
    fn from_reader(reader: &mut File) -> Result<ClassFileInterfaces, ClassLoadError> {
        let interfaces_count = reader.read_u16::<BigEndian>()?;
        let mut interfaces: Vec<u16> = vec![];
        for interface_index in 0 .. interfaces_count {
            let pool_index = reader.read_u16::<BigEndian>()?;
            println!("from_reader interface {} index {}", interface_index, pool_index);
            interfaces.push(pool_index);
        }
        Ok(ClassFileInterfaces {
            interfaces_count,
            interfaces
        })
    }
}

#[derive(Debug)]
struct AttributeInfo {
    attribute_name_index: u16,
    attribute_length: u32,
    info: Vec<u8>
}

impl AttributeInfo {
    fn from_reader(reader: &mut File) -> Result<AttributeInfo, ClassLoadError> {
        let attribute_name_index = reader.read_u16::<BigEndian>()?;
        let attribute_length = reader.read_u32::<BigEndian>()?;
        let mut info = vec![0u8; attribute_length as usize];
        reader.read_exact(&mut info)?;
        Ok(AttributeInfo {
            attribute_name_index,
            attribute_length,
            info
        })
    }

    fn from_cursor(reader: &mut Cursor<&&Vec<u8>>) -> Result<AttributeInfo, ClassLoadError> {
        let attribute_name_index = reader.read_u16::<BigEndian>()?;
        let attribute_length = reader.read_u32::<BigEndian>()?;
        let mut info = vec![0u8; attribute_length as usize];
        reader.read_exact(&mut info)?;
        Ok(AttributeInfo {
            attribute_name_index,
            attribute_length,
            info
        })
    }
}

#[derive(Debug)]
struct ClassFileAttributes {
    attributes_count: u16,
    attributes: Vec<AttributeInfo>
}

impl ClassFileAttributes {
    fn from_reader(reader: &mut File) -> Result<ClassFileAttributes, ClassLoadError> {
        let attributes_count = reader.read_u16::<BigEndian>()?;
        let mut attributes: Vec<AttributeInfo> = vec![];
        for attribute_index in 0 .. attributes_count {
            let info = AttributeInfo::from_reader(reader)?;
            println!("from_reader attribute {} info {:?}", attribute_index, info);
            attributes.push(info);
        }
        println!("from_reader attributes_count {} attributes {}", attributes_count, attributes.len());
        Ok(ClassFileAttributes {
            attributes_count,
            attributes
        })
    }
}

#[derive(Debug)]
struct FieldInfo {
    access_flags: Vec<AccessFlags>,
    name_index: u16,
    description_index: u16,
    attributes: ClassFileAttributes
}

impl FieldInfo {
    fn from_reader(reader: &mut File) -> Result<FieldInfo, ClassLoadError> {
        let access_flags = AccessFlags::from_reader(reader, false)?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let description_index = reader.read_u16::<BigEndian>()?;
        let attributes = ClassFileAttributes::from_reader(reader)?;
        println!("from_reader field name {} description {} attributes {} access {:?}", name_index, description_index, attributes.attributes_count, access_flags);
        Ok(FieldInfo {
            access_flags,
            name_index,
            description_index,
            attributes
        })
    }
}

struct ClassFileFields {
    fields_count: u16,
    fields: Vec<FieldInfo>
}

impl ClassFileFields {
    fn from_reader(reader: &mut File) -> Result<ClassFileFields, ClassLoadError> {
        let fields_count = reader.read_u16::<BigEndian>()?;
        let mut fields: Vec<FieldInfo> = vec![];
        for field_index in 0 .. fields_count {
            let field_info = FieldInfo::from_reader(reader)?;
            println!("from_reader field {} field_info {:?}", field_index, field_info);
            fields.push(field_info);
        }
        Ok(ClassFileFields {
            fields_count,
            fields
        })
    }
}

#[derive(Debug)]
struct MethodInfo {
    access_flags: Vec<AccessFlags>,
    name_index: u16,
    description_index: u16,
    attributes: ClassFileAttributes
}

impl MethodInfo {
    fn from_reader(reader: &mut File) -> Result<MethodInfo, ClassLoadError> {
        let access_flags = AccessFlags::from_reader(reader, false)?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let description_index = reader.read_u16::<BigEndian>()?;
        let attributes = ClassFileAttributes::from_reader(reader)?;
        println!("from_reader method name {} description {} attributes {} access {:?}", name_index, description_index, attributes.attributes_count, access_flags);
        Ok(MethodInfo {
            access_flags,
            name_index,
            description_index,
            attributes
        })
    }
}

#[derive(Debug)]
struct ClassFileMethods {
    methods_count: u16,
    methods: Vec<MethodInfo>
}

impl ClassFileMethods {
    fn from_reader(reader: &mut File) -> Result<ClassFileMethods, ClassLoadError> {
        let methods_count = reader.read_u16::<BigEndian>()?;
        let mut methods: Vec<MethodInfo> = vec![];
        for method_index in 0 ..methods_count {
            let method_info = MethodInfo::from_reader(reader)?;
            println!("from_reader method {} method_info {:?}", method_index, method_info);
            methods.push(method_info);
        }
        Ok(ClassFileMethods {
            methods_count,
            methods
        })
    }
}

struct ClassFileHeader {
    magic: u32,
    version_minor: u16,
    version_major: u16,
    constant_pool: ClassFileConstantPool,
    access_flags: Vec<AccessFlags>,
    this_class: u16,
    super_class: u16,
    interfaces: ClassFileInterfaces,
    fields: ClassFileFields,
    methods: ClassFileMethods,
    attributes: ClassFileAttributes
}

impl ClassFileHeader {
    fn from_reader(reader: &mut File) -> Result<ClassFileHeader, ClassLoadError> {
        let header = ClassFileHeader {
            magic: reader.read_u32::<BigEndian>()?,
            version_minor: reader.read_u16::<BigEndian>()?,
            version_major: reader.read_u16::<BigEndian>()?,
            constant_pool: ClassFileConstantPool::from_reader(reader)?,
            access_flags: AccessFlags::from_reader(reader, false)?,
            this_class: reader.read_u16::<BigEndian>()?,
            super_class: reader.read_u16::<BigEndian>()?,
            interfaces: ClassFileInterfaces::from_reader(reader)?,
            fields: ClassFileFields::from_reader(reader)?,
            methods: ClassFileMethods::from_reader(reader)?,
            attributes: ClassFileAttributes::from_reader(reader)?
        };
        if header.magic != 0xCAFEBABE {
            return Err(ClassLoadError::MagicMismatch(header.magic))
        }
        if header.version_major < 51 || header.version_major > 60 {
            return Err(ClassLoadError::VersionUnsupported(header.version_major, header.version_minor))
        }
        println!("from_reader access_flags {:?}", header.access_flags);
        println!("from_reader this_class {}", header.this_class);
        println!("from_reader super_class {}", header.super_class);
        Ok(header)
    }
}

struct ClassReader {
    header: ClassFileHeader
}

impl ClassReader {
    fn new(header: ClassFileHeader) -> Self {
        ClassReader{header}
    }

    fn get_constant_value(&self, key: usize) -> Option<&ConstantPoolTag> {
        self.header.constant_pool.constant_pool.get(key - 1)
    }

    fn get_constant_utf8(&self, index: usize) -> Option<&String> {
        if let ConstantPoolTag::Utf8(_, _, value) = self.get_constant_value(index)? {
            return Some(value)
        }
        None
    }

    fn get_attribute_value(&self, attribute: &AttributeInfo) -> Result<(&String, AttributeValue), ClassLoadError> {
        if let Some(name) = self.get_constant_utf8(attribute.attribute_name_index as usize) {
            return Ok((name, AttributeValue::from_name_and_info(name, &attribute.info)?))
        }
        Err(ClassLoadError::ConstantPoolMissing(attribute.attribute_name_index))
    }

    fn get_attributes_map(&self) -> HashMap<&String, AttributeValue> {
        let mut map: HashMap<_, _> = HashMap::new();
        for attribute in &self.header.attributes.attributes {
            let (name, value) = self.get_attribute_value(attribute).unwrap();
            map.insert(name, value);
        }
        map
    }

    fn get_class_attribute(&self, name: &str) -> Result<AttributeValue, ClassLoadError> {
        for attribute in &self.header.attributes.attributes {
            let (attr_name, value) = self.get_attribute_value(attribute)?;
            if name == attr_name {
                return Ok(value)
            }
        }
        Err(ClassLoadError::AttributeMissing(name.to_string()))
    }

    fn get_source_file(&self) -> Result<&String, ClassLoadError> {
        let key = "SourceFile";
        match self.get_class_attribute(key)? {
            AttributeValue::SourceFile(name_index) => self.get_constant_utf8(name_index as usize).ok_or(ClassLoadError::AttributeMissing(key.to_string())),
            x => Err(ClassLoadError::AttributeTypeMismatch(key.to_string(), format!("{:?}", x).to_string()))
        }
    }

    fn get_class_name(&self) -> Result<&String, ClassLoadError> {
        let entry = self.get_constant_value(self.header.this_class as usize)
            .ok_or(ClassLoadError::ConstantPoolMissing(self.header.this_class))?;
        if let ConstantPoolTag::Class(class_index) = entry {
            match self.get_constant_value(class_index.clone() as usize)
                .ok_or(ClassLoadError::ConstantPoolMissing(class_index.clone()))? {
                ConstantPoolTag::Utf8(_, _, name) => Ok(name),
                ConstantPoolTag::NameAndType(name_index, _) => {
                    Ok(self.get_constant_utf8(name_index.clone() as usize)
                        .ok_or(ClassLoadError::ConstantPoolMissing(name_index.clone()))?)
                },
                x => Err(ClassLoadError::ConstantPoolTypeMismatch("NameAndType".to_string(), format!("{:?}", x)))
            }
        } else {
            Err(ClassLoadError::ConstantPoolTypeMismatch("Class".to_string(), format!("{:?}", entry)))
        }
    }

    fn get_methods(&self) -> Result<HashMap<String, Method>, ClassLoadError> {
        let mut map: HashMap<String, Method> = HashMap::new();
        for method in &self.header.methods.methods {
            let name_index = method.name_index;
            let description_index = method.description_index;
            let method_name = self.get_constant_utf8(name_index as usize)
                .ok_or(ClassLoadError::ConstantPoolMissing(name_index))?;
            let description = self.get_constant_utf8(description_index as usize)
                .ok_or(ClassLoadError::ConstantPoolMissing(description_index))?;
            let mut method_code: Vec<u8> = Vec::new();
            for attribute in &method.attributes.attributes {
                let attribute_name = self.get_constant_utf8(attribute.attribute_name_index as usize)
                    .ok_or(ClassLoadError::ConstantPoolMissing(name_index))?;
                let attribute_value = AttributeValue::from_name_and_info(attribute_name, &attribute.info)?;
                println!("method {} : {} attribute {} length {} value {:?}", method_name, description, attribute_name, attribute.attribute_length, attribute_value);
                match attribute_value {
                    AttributeValue::Code(_, _, _, code, ..) => method_code = code,
                    _ => {}
                }
            }
            map.insert(method_name.clone(), Method {
                method_name: method_name.clone(),
                code: method_code
            });
        }
        Ok(map)
    }
}

#[derive(Debug)]
pub struct Method {
    pub method_name: String,
    pub code: Vec<u8>
}

#[derive(Debug)]
pub struct Class {
    class_name: String,
    source_file_name: String,
    methods: HashMap<String, Method>
}

impl Class {
    fn from_header(header: ClassFileHeader) -> Result<Self, ClassLoadError> {
        let reader = ClassReader::new(header);
        let source_file_name = reader.get_source_file()?;
        let class_name = reader.get_class_name()?;

        let methods = reader.get_methods()?;
        Ok(Class {
            class_name: class_name.clone(),
            source_file_name: source_file_name.clone(),
            methods: methods
        })
    }

    pub fn get_main(&self) -> Option<&Method> {
        return self.methods.get("main");
    }
 }

pub struct Loader {

}

impl Loader {
    pub fn load_from_file(&self, file_path: &str) -> Result<Class, ClassLoadError>{
        let mut file = File::open(file_path)?;
        self.load_from_reader(&mut file)
    }

    pub fn load_from_reader(&self, reader: &mut File) -> Result<Class, ClassLoadError> {
        let header = ClassFileHeader::from_reader(reader)?;
        Class::from_header(header)
    }
}

