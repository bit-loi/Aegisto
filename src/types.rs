use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SectionInfo {
    pub name: String,
    pub size: u64,
    pub virtual_address: u64,
    pub flags: String,
}

#[derive(Debug, Serialize)]
pub struct ImportInfo {
    pub name: String,
    pub library: String,
}

#[derive(Debug, Serialize)]
pub struct InstructionInfo {
    pub address: u64,
    pub mnemonic: String,
    pub operands: String,
}

#[derive(Debug, Serialize)]
pub struct StringMatch {
    pub offset: usize,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    pub file_path: String,
    pub format: String,
    pub entry_point: u64,
    pub sections: Vec<SectionInfo>,
    pub imports: Vec<ImportInfo>,
    pub instructions: Vec<InstructionInfo>,
    pub strings: Vec<StringMatch>,
}
