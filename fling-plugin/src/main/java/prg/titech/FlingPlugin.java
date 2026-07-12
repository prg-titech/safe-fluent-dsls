package prg.titech;

import org.gradle.api.Plugin;
import org.gradle.api.Project;
import org.gradle.api.plugins.JavaPluginExtension;
import org.gradle.api.tasks.SourceSet;
import org.gradle.api.tasks.SourceSetContainer;
import org.gradle.api.tasks.TaskProvider;


@SuppressWarnings("unused") public abstract class FlingPlugin implements Plugin<Project> {

    @Override
    public void apply(Project target) {
        SourceSetContainer sourceSets =
                target.getExtensions()
                        .getByType(JavaPluginExtension.class)
                        .getSourceSets();
        SourceSet grammar = sourceSets.create("grammar");

        TaskProvider<GenerateAPITask> taskProvider = target.getTasks().register(
                "generateFluentAPIs",
                GenerateAPITask.class,
                task -> {
                    task.dependsOn("grammarClasses");
                    task.getRuntimeClasspath().from(grammar.getRuntimeClasspath());
                    task.getClassesDir().from(grammar.getOutput().getClassesDirs());
                    task.getOutputDirectory().convention(
                            target.getLayout().getBuildDirectory().dir("generated/sources/fling/main/java")
                    );
                }
        );

        SourceSet main = sourceSets.getByName("main");
        main.getJava().srcDir(
                taskProvider.get().getOutputDirectory()
        );
    }

}
