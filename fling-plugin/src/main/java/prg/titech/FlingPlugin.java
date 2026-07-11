package prg.titech;

import org.gradle.api.Plugin;
import org.gradle.api.Project;

@SuppressWarnings("unused") public abstract class FlingPlugin implements Plugin<Project> {

    @Override
    public void apply(Project target) {
        target.getTasks().register(
                "generateAPITaskInFlingPlugin",
                GenerateAPITask.class,
                task -> {
                    task.getOutputFile().convention(
                            target.getLayout().getProjectDirectory().file("myfile.txt")
                    );
                    task.getFileText().set("HELLO FROM MY CONVENTION PLUGIN");
                }
        );
    }

}
