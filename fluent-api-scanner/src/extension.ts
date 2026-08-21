import * as vscode from 'vscode';

let diagnosticCollection: vscode.DiagnosticCollection;

const selector = { language: 'java', scheme: 'file' }; // register for all Java documents from the local file system

type Request = {
	raw_source_file: string,
	host_language: string,
	embedded_language: string,
	fluent_api: string
}

type Position = {
	line: number,
	column: number
}

function toVscodePosition(pos: Position): vscode.Position {
	return new vscode.Position(pos.line - 1, pos.column - 1);
}

function right(pos: Position): Position {
	return {
		line: pos.line,
		column: pos.column + 1
	};
}

type Range = {
	begin: Position,
	end: Position
};

function toVscodeRange(range: Range): vscode.Range {
	return new vscode.Range(toVscodePosition(range.begin), toVscodePosition(right(range.end)));
}

type Token = {
	image: string,
	range: Range
};

type ParseError = {
	source_token: Token,
	target_token: Token,
	message: string
};

type Analysis = {
	parse_errors: ParseError[]
};

function standardRequest(raw_file: string): Request {
	return {
		raw_source_file: raw_file,
		host_language: "java",
		embedded_language: "sql",
		fluent_api: "prg.titech.sql.Query"
	};
}

async function parseChains(raw_file: string): Promise<Analysis> {
	const response = await fetch("http://localhost:8080/analyze", {
		method: "POST",
		headers: {
			Accept: "application/json",
			"Content-Type": "application/json"
		},
		body: JSON.stringify(standardRequest(raw_file))
	});
	if (response.ok) {
		return await response.json() as Analysis;
	} else {
		throw new Error("Unable to access backend");
	}
}

function onOpen(document: vscode.TextDocument) {
	if (vscode.languages.match(selector, document) === 0) {
		// Our extension only concerns java files
		return;
	} 

	parseChains(document.getText()).then(
		(analysis) => {
			const diagnostics: vscode.Diagnostic[] = analysis.parse_errors.map(error => {
				return new vscode.Diagnostic(toVscodeRange(error.source_token.range), error.message);
			});
			diagnosticCollection.set(document.uri, diagnostics);
		},
		(reason) => {}
	);
}

function onChange(event: vscode.TextDocumentChangeEvent) {
	const document = event.document;
	onOpen(document);
}

const disposeDocumentListener = vscode.workspace.onDidChangeTextDocument(onChange);
const disposeOpenListener = vscode.workspace.onDidOpenTextDocument(onOpen);

export function activate(context: vscode.ExtensionContext) {
	diagnosticCollection = vscode.languages.createDiagnosticCollection('java');
    context.subscriptions.push(diagnosticCollection);
	console.log('Congratulations, your extension "fluent-api-scanner" is now active!');
}

// This method is called when your extension is deactivated
export function deactivate() {
	disposeDocumentListener.dispose();
	disposeOpenListener.dispose();
}
