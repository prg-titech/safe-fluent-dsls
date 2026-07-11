package prg.titech;

import org.apache.commons.io.FileUtils;
import org.gradle.testkit.runner.BuildResult;
import org.gradle.testkit.runner.GradleRunner;
import org.gradle.testkit.runner.TaskOutcome;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.net.URISyntaxException;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import java.util.Objects;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class FlingPluginTest {
    private static final Path projectDir = Path.of("build/test-project");
    private static final Path grammarDir = projectDir.resolve("src/grammar/java");
    private static final Path generatedSourcesDir = projectDir.resolve("build/generated/sources/fling/main/java");

    @BeforeAll
    static void deleteTestProject() throws IOException {
        FileUtils.deleteDirectory(projectDir.toFile());
    }

    @Test
    void createsFile() throws IOException {
        Files.createDirectories(projectDir);
        Files.writeString(projectDir.resolve("settings.gradle"), "");
        Path pluginJar = Path.of("build/libs")
                .resolve("fling-plugin-0.1-SNAPSHOT.jar")   // or whatever your archive name is
                .toAbsolutePath();

        Files.writeString(projectDir.resolve("build.gradle"), String.format("""
            plugins {
                id("java")
                id("prg.titech.fling-plugin")
            }
            
            dependencies {
                "grammarImplementation"(files("%s"))
            }
            """, pluginJar));
        copyResources("input", grammarDir);

        BuildResult result = GradleRunner.create()
                .withProjectDir(projectDir.toFile())
                .withPluginClasspath()
                .withArguments("generateFluentAPIs")
                .build();

        assertEquals(
                TaskOutcome.SUCCESS,
                Objects.requireNonNull(result.task(":generateFluentAPIs")).getOutcome()
        );

        Path outputAPI = generatedSourcesDir.resolve("Demo.java");
        assertTrue(Files.exists(outputAPI));
        Path outputAST = generatedSourcesDir.resolve("DemoAST.java");
        assertTrue(Files.exists(outputAST));
        Path outputCompiler = generatedSourcesDir.resolve("DemoCompiler.java");
        assertTrue(Files.exists(outputCompiler));
    }

    @SuppressWarnings("all") private static void copyResources(String resourceDir, Path destination) throws IOException {
        URL url = FlingPluginTest.class.getClassLoader().getResource(resourceDir);
        if (url == null) {
            throw new IllegalArgumentException("Resource not found: " + resourceDir);
        }

        Path source;
        try {
            source = Paths.get(url.toURI());
        } catch (URISyntaxException e) {
            throw new IOException(e);
        }

        try (Stream<Path> stream = Files.walk(source)) {
            stream.forEach(path -> {
                try {
                    Path target = destination.resolve(source.relativize(path));
                    if (Files.isDirectory(path)) {
                        Files.createDirectories(target);
                    } else {
                        Files.createDirectories(target.getParent());
                        Files.copy(path, target, StandardCopyOption.REPLACE_EXISTING);
                    }
                } catch (IOException e) {
                    throw new UncheckedIOException(e);
                }
            });
        }
    }
}
