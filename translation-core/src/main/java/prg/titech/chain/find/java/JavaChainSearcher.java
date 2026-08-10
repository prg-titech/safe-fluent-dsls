package prg.titech.chain.find.java;

import com.github.javaparser.StaticJavaParser;
import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.Node;
import prg.titech.chain.Chain;

import java.io.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.NoSuchElementException;
import java.util.Optional;
import java.util.Set;

public class JavaChainSearcher {

    public static List<Chain> findChains(File file, Set<String> entryMethods) throws IOException {
        try (FileInputStream input = new FileInputStream(file)) {
            return findChains(input, entryMethods);
        }
    }

    public static List<Chain> findChains(String input, Set<String> entryMethods) {
        return findChains(new ByteArrayInputStream(input.getBytes()), entryMethods);
    }

    public static List<Chain> findChains(Path path, Set<String> entryMethods) throws IOException {
        return findChains(Files.newInputStream(path), entryMethods);
    }

    public static List<Chain> findChains(InputStream input, Set<String> entryMethods) {
        CompilationUnit cu = StaticJavaParser.parse(input);
        List<Chain> foundChains = JavaChainSearchVisitor.findChains(cu);
        return foundChains.stream()
                .filter(c -> entryMethods.contains(c.getCalls().getFirst().getMethodName().toString()))
                .map(JavaExprReplaceVisitor::replaceIn)
                .filter(Optional::isPresent)
                .map(Optional::get)
                .toList();
    }

    public static Chain expectChain(Node n) throws NoSuchElementException {
        List<Chain> chains = JavaChainSearchVisitor.findChains(n);
        return chains.getFirst();
    }
}
