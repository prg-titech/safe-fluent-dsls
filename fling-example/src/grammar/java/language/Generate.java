package language;

import com.google.googlejavaformat.java.Formatter;
import com.google.googlejavaformat.java.FormatterException;

import java.io.IOException;
import java.nio.file.*;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Map;

public class Generate {
    private static final Path PATH = Paths.get("./src/main/java/example/generated/").toAbsolutePath();
    private static final Map<String, String> files;

    static {
        Map<String, String> $ = new LinkedHashMap<>();
        $.put("BalancedParentheses", BalancedParentheses.jm.apiClass);
        $.put("BalancedParenthesesAST", BalancedParentheses.jm.astClass);
        $.put("BalancedParenthesesCompiler", BalancedParentheses.jm.astCompilerClass);
        files = $;
    }

    public void compile() throws IOException, FormatterException {
        System.out.printf("project path: %s%n", PATH);
        if (!Files.exists(PATH)) {
            Files.createDirectory(PATH);
            System.out.printf("directory %s created successfully%n", PATH);
        }

        Formatter formatter = new Formatter();

        for(Map.Entry<String, String> file : files.entrySet()) {
            Path filePath = PATH.resolve(file.getKey() + ".java");
            if (Files.exists(filePath)) {
                Files.delete(filePath);
            }

            Files.write(filePath, Collections.singleton(formatter.formatSource((String)file.getValue())), StandardOpenOption.CREATE, StandardOpenOption.WRITE);
            System.out.printf("file %s.java written successfully.%n", file.getKey());
        }

    }

    public static void main(String[] args) throws FormatterException, IOException {
        new Generate().compile();
    }
}
