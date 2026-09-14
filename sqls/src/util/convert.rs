use tower_lsp_server::ls_types;
use texter::lsp_types;
use texter::tree_sitter;

pub fn convert_text_document_content_change_event(from: ls_types::TextDocumentContentChangeEvent) -> lsp_types::TextDocumentContentChangeEvent {
    lsp_types::TextDocumentContentChangeEvent { 
        range: from.range.map(convert_range), 
        range_length: from.range_length, 
        text: from.text 
    }
}

pub fn convert_range(from: ls_types::Range) -> lsp_types::Range {
    lsp_types::Range { 
        start: convert_position(from.start), 
        end: convert_position(from.end), 
    }
}

pub fn convert_ts_range(from: tree_sitter::Range) -> ls_types::Range {
    ls_types::Range {
        start: convert_point(from.start_point),
        end: convert_point(from.end_point)
    }
}

pub fn convert_position(from: ls_types::Position) -> lsp_types::Position {
    lsp_types::Position { 
        line: from.line, 
        character: from.character,
    }
}

pub fn convert_point(from: tree_sitter::Point) -> ls_types::Position {
    ls_types::Position { 
        line: from.row as u32, 
        character: from.column as u32
    }
}