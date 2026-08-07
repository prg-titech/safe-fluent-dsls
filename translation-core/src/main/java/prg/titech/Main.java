package prg.titech;

import net.sf.jsqlparser.parser.ParseException;
import prg.titech.chain.Chain;
import prg.titech.chain.find.java.JavaChainSearcher;
import prg.titech.chain.translate.TokenList;
import prg.titech.sql.analyze.SQLAnalyzer;
import prg.titech.sql.translate.SQLTranslator;

import java.io.IOException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Optional;
import java.util.Set;

public class Main {

    private static final Path testPath = Paths.get("src", "test", "java", "prg", "titech", "TestFixtures.java");

    public static void main(String[] args) throws IOException {
        List<Chain> foundChains = JavaChainSearcher.findChains(testPath, Set.of("select"));
        for (Chain c : foundChains) {
            System.out.println("Found chain: " + c);
            TokenList translation = SQLTranslator.translateTokens(c);
            System.out.println("Translation: " + translation);
            Optional<ParseException> parseError = SQLAnalyzer.analyze(translation);
            parseError.ifPresentOrElse(
                    e -> {
                        System.out.println("Parse error found!");
                        System.out.println("\t" + e);
                    },
                    () -> System.out.println("No errors found!")
            );
            System.out.println("---------------");
        }
    }
}
