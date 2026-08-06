package prg.titech;

import com.github.javaparser.StaticJavaParser;
import com.github.javaparser.ast.CompilationUnit;
import prg.titech.chain.Chain;
import prg.titech.chain.find.java.JavaChainSearchVisitor;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.List;

public class Main {

    public static void main(String[] args) throws IOException {
        CompilationUnit cu = StaticJavaParser.parse(Files.newInputStream(Paths.get("src", "test", "java", "prg", "titech", "TestFixtures.java").toAbsolutePath()));

        List<Chain> foundChains = JavaChainSearchVisitor.findChains(cu);
        System.out.println(foundChains);
    }
}
