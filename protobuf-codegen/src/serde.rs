use crate::code_writer::CodeWriter;
use crate::customize::Customize;

/// Write serde attr according to specified codegen option.
pub fn write_serde_attr(w: &mut CodeWriter, customize: &Customize, attr: &str) {
    if customize.serde_derive.unwrap_or(false) {
        if let Some(ref cfg) = customize.serde_derive_cfg {
            w.write_line(&format!("#[cfg_attr({}, {})]", cfg, attr));
        } else {
            w.write_line(&format!("#[{}]", attr));
        }
    }
}

pub fn write_serde_attr_repr(w: &mut CodeWriter, customize: &Customize) {
    if customize.serde_derive.unwrap_or(false) {
        write_serde_attr(w, customize, "derive(Serialize_repr, Deserialize_repr)");
    }
}

pub fn write_serde_repr(w: &mut CodeWriter, customize: &Customize) {
    if customize.serde_derive.unwrap_or(false) {
        // Enums are numbers in protobuf
        w.write_line(&format!("#[repr(i16)]"))
    }
}
