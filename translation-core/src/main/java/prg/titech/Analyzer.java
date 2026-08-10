package prg.titech;

import com.google.common.math.IntMath;
import picocli.CommandLine;
import prg.titech.chain.Chain;
import prg.titech.chain.find.java.JavaChainSearcher;
import prg.titech.chain.projection.ParseError;
import prg.titech.chain.token.Position;
import prg.titech.chain.token.Range;
import prg.titech.sql.analyze.SQLAnalyzer;
import prg.titech.sql.translate.SQLTranslator;

import java.io.File;
import java.io.IOException;
import java.math.RoundingMode;
import java.nio.file.Files;
import java.util.List;
import java.util.Set;

@CommandLine.Command(name = "fluent-api-analyzer", mixinStandardHelpOptions = true, version = "0.1",
        description = "Analyzes a given file for correct usage of pre-defined fluent SQL API.")
public class Analyzer implements Runnable {

    @CommandLine.Parameters(index = "0")
    File sourceFile;

    @Override
    public void run() {
        List<String> sourceFile;
        try {
            sourceFile = readFile(this.sourceFile);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        List<Chain> foundChains = JavaChainSearcher.findChains(String.join("\n", sourceFile), Set.of("select"));
        List<ParseError> parseErrors = foundChains.stream()
                .map(SQLTranslator::translate)
                .flatMap(t -> SQLAnalyzer.parse(t).stream())
                .toList();

        for (ParseError error : parseErrors) {
            System.out.println(reportParseError(error, sourceFile));
        }
    }

    public static void main(String... args) {
        int exitCode = new CommandLine(new Analyzer()).execute(args);
        System.exit(exitCode);
    }

    private List<String> readFile(File file) throws IOException {
        return Files.readAllLines(file.toPath());
    }

    private String reportParseError(ParseError error, List<String> sourceFile) {
        Range targetRange = error.getSourceToken().getRange().orElseThrow();
        int highlightLine = targetRange.begin().line() - Position.HOME.line();
        int highlightBegin = Math.max(0, highlightLine - 2);
        int highlightEnd = Math.min(sourceFile.size() - 1, highlightLine + 2);
        int indexWidth = IntMath.log10(sourceFile.size(), RoundingMode.CEILING);

        StringBuilder sb = new StringBuilder();
        for (int i = highlightBegin; i <= highlightLine; i++) {
            pasteLine(sb, sourceFile.get(i), i, indexWidth);
        }
        attachMessageToSection(sb, error.getMessage(), targetRange.begin().column() - Position.HOME.column(), targetRange.end().column() - Position.HOME.column(), indexWidth);
        for (int i = highlightLine + 1; i <= highlightEnd; i++) {
            pasteLine(sb, sourceFile.get(i), i, indexWidth);
        }
        return sb.toString();
    }

    private void pasteLine(StringBuilder sb, String line, int index, int indexWidth) {
        String indexAsString = Integer.toString(index + 1);
        sb.append(indexAsString);
        sb.repeat(" ", Math.max(1, indexWidth - indexAsString.length() + 1));
        sb.append(line);
        sb.append('\n');
    }

    private void attachMessageToSection(StringBuilder sb, String message, int begin, int end, int indexWidth) {
        List<String> messageLines = message.lines().toList();
        int messageBoxWidth = Math.max(end, messageLines.stream().map(String::length).max(Integer::compareTo).orElseThrow()) + 4;

        sb.repeat(" ", indexWidth);
        sb.append("+");
        sb.repeat("-", begin);
        sb.repeat("^", end - begin + 1);
        sb.repeat("-", messageBoxWidth - end - 3);
        sb.append("+");
        sb.append("\n");
        for (String line : messageLines) {
            sb.repeat(" ", indexWidth);
            sb.append("| ");
            sb.append(line);
            sb.repeat(" ", messageBoxWidth - line.length() - 3);
            sb.append("|\n");
        }
        sb.repeat(" ", indexWidth);
        sb.append("+");
        sb.repeat("-", messageBoxWidth - 2);
        sb.append("+\n");
    }
}
