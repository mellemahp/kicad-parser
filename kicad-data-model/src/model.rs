use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a `KiCad` Schematic file
///
/// Each component is based on the sections defined
/// here: <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic>
///
/// Data should be serialized out in the order defined here.
///
/// Note: Text_boxes is not found in the spec but was found in a number
/// of schematic files.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "kicad_sch")]
pub struct KiCadSchema {
    version: u32,
    generator: String,
    uuid: Uuid,
    paper: String,
    title_block: Option<TitleBlock>,
    lib_symbols: LibSymbols,
    #[serde(default, rename = "junction")]
    junctions: Vec<Junction>,
    #[serde(default, rename = "no_connect")]
    no_connect: Vec<NoConnect>,
    #[serde(default, rename = "bus_entry")]
    bus_entries: Vec<BusEntry>,
    #[serde(default, rename = "wire")]
    wires: Vec<WireOrBus>,
    #[serde(default, rename = "bus")]
    buses: Vec<WireOrBus>,
    #[serde(default, rename = "polyline", skip_serializing_if = "Vec::is_empty")]
    graphical_lines: Vec<Polyline>,
    #[serde(default, rename = "text")]
    graphical_text: Vec<Text>,
    #[serde(default, rename = "text_box")]
    text_boxes: Vec<TextBox>,
    #[serde(default, rename = "label")]
    local_labels: Vec<Label>,
    // TODO: GLOBAL LABEL <NOT ADDED>
    #[serde(default, rename = "hierarchical_label")]
    hierarchical_labels: Vec<HierarchicalLabel>,
    #[serde(default, rename = "symbol")]
    symbols: Vec<Symbol>,
    // TODO: Heirarchical sheets <NOT ADDED>
    sheet_instances: SheetInstances,
    embedded_fonts: Option<YesOrNo>,
}

impl KiCadSchema {
    /// Adds new graphical text to the schema
    pub fn add_text(&mut self, text: Text) {
        self.graphical_text.push(text);
    }

    /// Add a property to a symbol
    pub fn add_property(&mut self, symbol: &str, property: Property) {
        if let Some(symbol) = self.symbols.iter_mut().find(|s| s.lib_id == symbol) {
            symbol.properties.push(property);
        }
        // TODO: Option to bubble up to cli
    }

    /// Remove a property from a symbol
    pub fn remove_property(&mut self, symbol: &str, property_name: &str) {
        if let Some(symbol) = self.symbols.iter_mut().find(|s| s.lib_id == symbol) {
            symbol.properties.retain(|item| !(item.keys.1 == property_name));
        }
        // TODO: Option to bubble up to cli
    }
}

// ============================================================================
// Sections
// ============================================================================

// ==== Title block ====

/// Title block
///
/// See: <https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_title_block>
#[derive(Debug, Serialize, Deserialize)]
pub struct TitleBlock {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    company: Option<String>,
    // TODO: add comments <Skipped as seemed uncommon>
}

// ===== Symbol Library

/// Symbol library section containing one or more library symbols
///
/// See: <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_library_symbol_section>
#[derive(Debug, Serialize, Deserialize)]
struct LibSymbols {
    #[serde(default, rename = "symbol")]
    symbols: Vec<LibSymbol>,
}

/// Defines a library symbol
///
/// Note: Distinct from other symbol definition for now
/// although perhaps could be merged via flattening
#[derive(Debug, Serialize, Deserialize)]
struct LibSymbol {
    #[serde(rename = "")]
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    power: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pin_number: Option<PinNumbers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pin_names: Option<PinNames>,
    exclude_from_sim: YesOrNo,
    in_bom: YesOrNo,
    on_board: YesOrNo,
    in_pos_files: YesOrNo,
    duplicate_pin_numbers_are_jumpers: YesOrNo,
    #[serde(rename = "property")]
    properties: OneOrMulti<Property>,
    #[serde(rename = "symbol")]
    sub_symbols: OneOrMulti<SymbolDef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PinNumbers {
    #[serde(skip_serializing_if = "Option::is_none")]
    hide: Option<YesOrNo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PinNames {
    offset: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    hide: Option<YesOrNo>,
}


/// Represents a symbol geometry definition
///
/// TODO: Could make geometry an Enum
#[derive(Debug, Serialize, Deserialize)]
struct SymbolDef {
    #[serde(rename = "")]
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    polyline: Option<OneOrMulti<PolylineLib>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    arc: Option<OneOrMulti<Arc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pin: Option<OneOrMulti<PinLib>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    rectangle: Option<OneOrMulti<Rectangle>>
}

// NOTE: Annoyingly different than the later one...
// TODO: Maybe could merge
#[derive(Debug, Serialize, Deserialize)]
struct PolylineLib {
    pts: Pts,
    stroke: Stroke,
    fill: Fill,
}

#[derive(Debug, Serialize, Deserialize)]
struct Arc {
    start: Point,
    mid: Point,
    end: Point,
    stroke: Stroke,
    fill: Fill,
}

#[derive(Debug, Serialize, Deserialize)]
struct PinLib {
    #[serde(rename = "")]
    keys: (String, String),
    at: At,
    length: f32,
    name: PinIdent,
    number: PinIdent,
}

#[derive(Debug, Serialize, Deserialize)]
struct PinIdent {
    #[serde(rename = "")]
    name: String,
    effects: Effects,
}

#[derive(Debug, Serialize, Deserialize)]
struct Rectangle {
    start: Point,
    end: Point,
    stroke: Stroke,
    fill: Fill
}

// ===== Junction =====

/// Junction as defined here:
/// <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_junction_section>
#[derive(Debug, Serialize, Deserialize)]
pub struct Junction {
    at: At,
    diameter: f32,
    color: Rgba,
    uuid: Uuid,
}

/// Represents a Red, Green, Blue, Alpha, color
/// value
#[derive(Debug, Serialize, Deserialize)]
pub struct Rgba(pub u32, pub u32, pub u32, pub u32);

// ===== No Connect Section  =====

/// No connect as defined here:
/// <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_no_connect_section>
#[derive(Debug, Serialize, Deserialize)]
struct NoConnect {
    at: At,
    uuid: Uuid,
}

// ===== Bus Section =====

/// Bus entry as defined here:
/// <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_bus_entry_section>
#[derive(Debug, Serialize, Deserialize)]
struct BusEntry {
    at: At,
    size: Size,
    stroke: Stroke,
    uuid: Uuid,
}

// ==== Wire and Bus ====

/// Wire or Bus as defined here:
/// <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_wire_and_bus_section>
#[derive(Debug, Serialize, Deserialize)]
struct WireOrBus {
    pts: Pts,
    stroke: Stroke,
    uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pts {
    pub xy: Vec<Point>,
}

// ==== Polyline section ====

/// Polyline section as defined in spec.
///
/// NOTE: Not quite the same as polyline definition
/// in symbol lib
#[derive(Debug, Serialize, Deserialize)]
struct Polyline {
    pts: Pts,
    stroke: Stroke,
    uuid: Uuid,
}

// ===== Text =====

/// graphical Text Section
///
/// See: <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_graphical_text_section>
#[derive(Debug, Serialize, Deserialize)]
pub struct Text {
    // Content has no key!
    #[serde(rename = "")]
    pub content: String,
    pub at: At,
    pub effects: Effects,
    pub uuid: Uuid,
}

/// Text Box section
///
/// TODO: Could possibly have `Text` data flattened in
#[derive(Debug, Serialize, Deserialize)]
struct TextBox {
    // Content has no key!
    #[serde(rename = "")]
    content: String,
    exclude_from_sim: YesOrNo,
    at: At,
    size: Size,
    margins: Margins,
    stroke: Stroke,
    fill: Fill,
    effects: Effects,
    uuid: Uuid,
}

/// Label Section
///
/// See: <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_local_label_section>
#[derive(Debug, Serialize, Deserialize)]
struct Label {
    // Content has no key!
    #[serde(rename = "")]
    content: String,
    at: At,
    effects: Effects,
    uuid: Uuid,
}

#[allow(dead_code)]
struct GlobalLabel {
    // TODO: Didnt see any examples of so skipping for now.
}

#[derive(Debug, Serialize, Deserialize)]
struct HierarchicalLabel {
    // Content has no key!
    #[serde(rename = "")]
    content: String,
    shape: String,
    at: At,
    effects: Effects,
    uuid: Uuid,
}

// ===== Symbol Section ======

/// Symbol Section
///
/// See: <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_symbol_section>
#[derive(Debug, Serialize, Deserialize)]
struct Symbol {
    lib_id: String,
    at: At,
    unit: u32,
    body_style: u32,
    exclude_from_sim: YesOrNo,
    in_bom: YesOrNo,
    on_board: YesOrNo,
    in_pos_files: YesOrNo,
    dnp: YesOrNo,
    uuid: Uuid,
    #[serde(default, rename = "property")]
    properties: Vec<Property>,
    #[serde(rename = "pin")]
    pins: OneOrMulti<Pin>,
    instances: Projects,
}

#[derive(Debug, Serialize, Deserialize)]
struct Projects {
    #[serde(default, rename = "project")]
    values: Vec<Project>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SheetInstances {
    #[serde(rename = "path")]
    paths: SheetPaths,
}

// ==== root instances section

/// Root instances section
///
/// See: <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_root_sheet_instance_section>
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum SheetPaths {
    One(SheetPath),
    Multi(Vec<SheetPath>),
}

#[derive(Debug, Serialize, Deserialize)]
struct SheetPath {
    #[serde(rename = "")]
    name: String,
    page: String,
}

// ============================================================================
// Properties
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "")]
    pub keys: (String, String),
    pub at: At,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide: Option<YesOrNo>,
    pub show_name: YesOrNo,
    pub do_not_autoplace: YesOrNo,
    pub effects: Effects,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pin {
    #[serde(rename = "")]
    id: String,
    uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    #[serde(rename = "")]
    name: String,
    path: ProjectPath,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectPath {
    #[serde(rename = "")]
    path: String,
    reference: String,
    unit: u32,
}



// ============================================================================
// Common Models
// ============================================================================

/// X and Y location
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum At {
    WithRotation(f32, f32, f32),
    Standard(f32, f32),
}

/// Defines how bus entry is drawn
#[derive(Debug, Serialize, Deserialize)]
pub struct Stroke {
    width: f32,
    #[serde(rename = "type")]
    ty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Rgba>,
}

/// Represents a point in a point list
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Point {
    Coordinates(f64, f64),
}

/// Size of text or shape
#[derive(Debug, Deserialize, Serialize)]
pub struct Size(pub f32, pub f32);


/// Text effects
#[derive(Debug, Serialize, Deserialize)]
pub struct Effects {
    pub font: FontOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<OneOrMulti<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide: Option<YesOrNo>,
}

/// Options for font display
#[derive(Debug, Serialize, Deserialize)]
pub struct FontOptions {
    pub size: Size,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thickness: Option<f32>,
    // Catch anything else at end of block like "bold" or "italic".
    #[serde(default, rename = "", skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
}

/// Display fill setting
#[derive(Debug, Serialize, Deserialize)]
struct Fill {
    #[serde(rename = "type")]
    pub fill_type: FillType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FillType {
    None,
    Background,
    Color,
}


#[derive(Debug, Serialize, Deserialize)]
struct Margins(f32, f32, f32, f32);


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum YesOrNo {
    Yes,
    No,
}

/// Convenience enum for mapping single maps and vecs into a common type.
///
/// TODO: Could probably remove with some tweaks to parser, but fine for now.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMulti<T> {
    Multi(Vec<T>),
    One(T),
}
impl <T> OneOrMulti<T> {
    /// Get first element, matching vec api
    fn first(&self) -> Option<&T> {
        match self {
            OneOrMulti::Multi(t) => Some(&t[0]),
            OneOrMulti::One(t) => Some(t),
        }
    }

    /// Get value, matching vec api
    fn get(&self, index: usize) -> Option<&T> {
        match self {
            OneOrMulti::One(t) if index == 0 => Some(t),
            OneOrMulti::Multi(t) => t.get(index),
            _ => None
        }
    }
}
