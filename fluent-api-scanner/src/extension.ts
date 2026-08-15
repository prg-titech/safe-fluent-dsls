import * as vscode from 'vscode';

const tokenTypes = ['keyword'];
const tokenModifiers: string[] = [];
const legend = new vscode.SemanticTokensLegend(tokenTypes, tokenModifiers);

const provider: vscode.DocumentSemanticTokensProvider = {
  provideDocumentSemanticTokens(
    document: vscode.TextDocument
  ): vscode.ProviderResult<vscode.SemanticTokens> {
    // analyze the document and return semantic tokens
	const selectPositions: vscode.Position[] = [];
	document.getText().split("\n").forEach((line, index) => {
		const splitPoints: number[] = line.split("select").map(s => s.length);
		splitPoints.pop();
		let currentColumn = 0;
		splitPoints.forEach(length => {
			currentColumn += length;
			selectPositions.push(new vscode.Position(index, currentColumn));
			currentColumn += 6;
		});
	});

    const tokensBuilder = new vscode.SemanticTokensBuilder(legend);
	selectPositions.forEach(position => {
		tokensBuilder.push(
			new vscode.Range(position, new vscode.Position(position.line, position.character + 6)),
			"keyword",
			[]
		);
	});
    return tokensBuilder.build();
  }
};

const selector = { language: 'java', scheme: 'file' }; // register for all Java documents from the local file system

vscode.languages.registerDocumentSemanticTokensProvider(selector, provider, legend);

export function activate(context: vscode.ExtensionContext) {
	console.log('Congratulations, your extension "fluent-api-scanner" is now active!');
}

// This method is called when your extension is deactivated
export function deactivate() {}
