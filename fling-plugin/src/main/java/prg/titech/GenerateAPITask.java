package prg.titech;

import org.gradle.api.DefaultTask;
import org.gradle.api.file.RegularFileProperty;
import org.gradle.api.provider.Property;
import org.gradle.api.tasks.Input;
import org.gradle.api.tasks.OutputFile;
import org.gradle.api.tasks.TaskAction;

import java.io.IOException;
import java.nio.file.Files;

public abstract class GenerateAPITask extends DefaultTask {
    @Input
    public abstract Property<String> getFileText();

    @OutputFile
    public abstract RegularFileProperty getOutputFile();

    @TaskAction
    public void action() throws IOException {
        Files.writeString(
                getOutputFile().get().getAsFile().toPath(),
                getFileText().get()
        );
    }
}
