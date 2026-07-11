package prg.titech;

import org.gradle.testkit.runner.BuildResult;
import org.gradle.testkit.runner.GradleRunner;
import org.gradle.testkit.runner.TaskOutcome;
import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Objects;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class FlingPluginTest {
    Path projectDir = Path.of("build/test-project");

    @Test
    void createsFile() throws Exception {
        Files.createDirectories(projectDir);
        Files.writeString(projectDir.resolve("settings.gradle"), "");

        Files.writeString(projectDir.resolve("build.gradle"), """
            plugins {
                id 'prg.titech.fling-plugin'
            }
            """);

        BuildResult result = GradleRunner.create()
                .withProjectDir(projectDir.toFile())
                .withPluginClasspath()
                .withArguments("generateAPITaskInFlingPlugin")
                .build();

        assertEquals(
                TaskOutcome.SUCCESS,
                Objects.requireNonNull(result.task(":generateAPITaskInFlingPlugin")).getOutcome()
        );

        Path output = projectDir.resolve("myfile.txt");
        assertTrue(Files.exists(output));
        assertEquals(
                "HELLO FROM MY CONVENTION PLUGIN",
                Files.readString(output));
    }
}
