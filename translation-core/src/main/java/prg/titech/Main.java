package prg.titech;

import prg.titech.chain.Chain;
import prg.titech.chain.find.java.JavaChainSearcher;
import prg.titech.chain.projection.ParseError;
import prg.titech.chain.translate.Translation;
import prg.titech.sql.analyze.SQLAnalyzer;
import prg.titech.sql.translate.SQLTranslator;

import java.io.IOException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Set;

public class Main {

    private static final Path testPath = Paths.get("src", "test", "java", "prg", "titech", "TestFixtures.java");

    public static void main(String[] args) throws IOException {
        List<Chain> foundChains = JavaChainSearcher.findChains(testPath, Set.of("select"));
        for (Chain c : foundChains) {
            System.out.println("Found chain: " + c);
            Translation translation = SQLTranslator.translate(c);
            System.out.println("Translation Details:\n" + translation.toDebugString());
            List<ParseError> errors = SQLAnalyzer.parse(translation);
            for (ParseError e : errors) {
                System.out.println("Got parse error at " + e.targetToken().toDebugString() + ":\n" + e.message());
            }
            System.out.println("---------------");
        }
    }
}
