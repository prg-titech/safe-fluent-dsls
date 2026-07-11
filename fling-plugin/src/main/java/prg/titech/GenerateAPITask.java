package prg.titech;

import com.google.googlejavaformat.java.Formatter;
import com.google.googlejavaformat.java.FormatterException;
import fling.adapters.JavaMediator;
import org.gradle.api.DefaultTask;
import org.gradle.api.GradleException;
import org.gradle.api.file.ConfigurableFileCollection;
import org.gradle.api.file.DirectoryProperty;
import org.gradle.api.provider.Property;
import org.gradle.api.tasks.*;
import prg.titech.api.Grammar;

import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.net.MalformedURLException;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public abstract class GenerateAPITask extends DefaultTask {
    @Input
    public abstract Property<String> getClassName();

    @Classpath
    public abstract ConfigurableFileCollection getRuntimeClasspath();

    @OutputDirectory
    public abstract DirectoryProperty getOutputDirectory();

    @TaskAction
    public void action() throws IOException, FormatterException {
        List<URL> urls = getRuntimeClasspath()
                .getFiles()
                .stream()
                .map(file -> {
                    try {
                        return file.toURI().toURL();
                    } catch (MalformedURLException e) {
                        throw new RuntimeException(e);
                    }
                })
                .toList();
        for (URL url : urls) {
            System.out.println("DEBUG: " + url);
        }

        Class<?> clazz;
        Grammar grammar;
        try (URLClassLoader loader = new URLClassLoader(
                urls.toArray(URL[]::new),
                Grammar.class.getClassLoader())) {
            clazz = loader.loadClass(getClassName().get());
            grammar = (Grammar) clazz.getDeclaredConstructor().newInstance();
            if (!Grammar.class.isAssignableFrom(clazz)) {
                throw new GradleException(
                        clazz.getName() + " does not implement A");
            }
        } catch (ClassNotFoundException e) {
            throw new GradleException(String.format("Unable to find class %s in src/grammar/java", getClassName().get()), e);
        } catch (NoSuchMethodException | IllegalArgumentException e) {
            throw new GradleException(String.format("Class %s does not have a no-arg constructor", getClassName().get()), e);
        } catch (IllegalAccessException e) {
            throw new GradleException(String.format("Unable to access no-arg constructor of class %s", getClassName().get()), e);
        } catch (InstantiationException e) {
            throw new GradleException(String.format("Tried to instantiate abstract class %s. If this occurred for an automatically generated task, please send us a bug report!", getClassName().get()), new AssertionError("Class is not concrete.", e));
        } catch (InvocationTargetException e) {
            throw new GradleException(String.format("An error occurred during instantiation of class %s.", getClassName().get()), e);
        } catch (ClassCastException e) {
            throw new GradleException(String.format("Class %s does not implemented Grammar", getClassName().get()), e);
        }

        JavaMediator jm = grammar.getJavaMediator();
        Map<String, String> generatedSources = new HashMap<>(3);
        generatedSources.put(clazz.getName(), jm.apiClass);
        generatedSources.put(clazz.getName() + "AST", jm.astClass);
        generatedSources.put(clazz.getName() + "Compiler", jm.astCompilerClass);

        Path output = getOutputDirectory().get().getAsFile().toPath();
        Files.createDirectories(output);

        Formatter formatter = new Formatter();
        for (Map.Entry<String, String> file : generatedSources.entrySet()) {
            Path filePath = output.resolve(file.getKey() + ".java");
            Files.createDirectories(filePath.getParent());
            Files.writeString(filePath, formatter.formatSource(file.getValue()));
        }
    }
}
