package prg.titech;

import com.google.googlejavaformat.java.Formatter;
import com.google.googlejavaformat.java.FormatterException;
import fling.adapters.JavaMediator;
import org.gradle.api.DefaultTask;
import org.gradle.api.GradleException;
import org.gradle.api.file.ConfigurableFileCollection;
import org.gradle.api.file.DirectoryProperty;
import org.gradle.api.tasks.*;
import org.jspecify.annotations.NonNull;
import prg.titech.api.Grammar;

import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Modifier;
import java.net.MalformedURLException;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@CacheableTask
public abstract class GenerateAPITask extends DefaultTask {
    @Classpath
    public abstract ConfigurableFileCollection getClassesDir();

    @Classpath
    public abstract ConfigurableFileCollection getRuntimeClasspath();

    @OutputDirectory
    public abstract DirectoryProperty getOutputDirectory();

    @TaskAction
    public void action() throws IOException, FormatterException {
        URL[] urls = Stream.concat(
                Stream.of(getClassesDir().getSingleFile().toURI().toURL()),
                getRuntimeClasspath()
                .getFiles()
                .stream()
                .map(file -> {
                    try {
                        return file.toURI().toURL();
                    } catch (MalformedURLException e) {
                        throw new RuntimeException(e);
                    }
                }))
                .toArray(URL[]::new);

        Map<Class<?>, Grammar> grammars;
        try (URLClassLoader loader = new URLClassLoader(urls, Grammar.class.getClassLoader());
            Stream<Path> files = Files.walk(getClassesDir().getSingleFile().toPath())) {
            grammars = files.filter(p -> p.toString().endsWith(".class"))
                            .map(p -> {
                                try {
                                    return loadClass(loader, p);
                                } catch (MalformedURLException | ClassNotFoundException e) {
                                    throw new RuntimeException(e);
                                }
                            })
                            .filter(GenerateAPITask::isGrammarClass)
                            .collect(
                                Collectors.toMap(
                                        c -> c,
                                        GenerateAPITask::instantiateGrammar
                                )
                            );

        }

        Map<String, String> generatedSources = getGeneratedSources(grammars);
        Path output = getOutputDirectory().get().getAsFile().toPath();
        Files.createDirectories(output);
        Formatter formatter = new Formatter();
        for (Map.Entry<String, String> file : generatedSources.entrySet()) {
            Path filePath = resolveQualifiedName(output, file.getKey());
            Files.createDirectories(filePath.getParent());
            Files.writeString(filePath, formatter.formatSource(file.getValue()));
        }
    }

    private static @NonNull Map<String, String> getGeneratedSources(Map<Class<?>, Grammar> grammars) {
        Map<String, String> generatedSources = new HashMap<>(3 * grammars.size());
        for (Map.Entry<Class<?>, Grammar> grammarEntry : grammars.entrySet()) {
            JavaMediator jm = grammarEntry.getValue().getJavaMediator();
            generatedSources.put(grammarEntry.getKey().getName(), jm.apiClass);
            generatedSources.put(grammarEntry.getKey().getName() + "AST", jm.astClass);
            generatedSources.put(grammarEntry.getKey().getName() + "Compiler", jm.astCompilerClass);
        }
        return generatedSources;
    }

    private Class<?> loadClass(URLClassLoader loader, Path classPath) throws MalformedURLException, ClassNotFoundException {
        StringBuilder className = new StringBuilder();
        for (Path nameElement : getClassesDir().getSingleFile().toPath().relativize(classPath)) {
            if (nameElement.toString().endsWith(".class")) {
                className.append(nameElement.toString().replace(".class", ""));
            } else {
                className.append(nameElement);
                className.append('.');
            }
        }
        return loader.loadClass(className.toString());
    }

    private static boolean isGrammarClass(Class<?> clazz) {
        return clazz != null && isConcrete(clazz) && Grammar.class.isAssignableFrom(clazz) && hasNullaryConstructor(clazz);
    }

    private static Grammar instantiateGrammar(Class<?> clazz) {
        try {
            return (Grammar) clazz.getConstructor().newInstance();
        } catch (NoSuchMethodException e) {
            throw new AssertionError(String.format("Expected class %s to have a nullary constructor", clazz.getName()), e);
        } catch (IllegalAccessException e) {
            throw new GradleException(String.format("Unable to access constructor of class %s", clazz.getName()), e);
        } catch (InstantiationException e) {
            throw new AssertionError(String.format("Expected class %s to be concrete (non-abstract)", clazz.getName()), e);
        } catch (InvocationTargetException e) {
            throw new RuntimeException(e);
        }
    }

    private static boolean isConcrete(Class<?> clazz) {
        return (clazz.getModifiers() & (Modifier.ABSTRACT | Modifier.INTERFACE)) == 0;
    }

    private static boolean hasNullaryConstructor(Class<?> clazz) {
        try {
            clazz.getConstructor();
            return true;
        } catch (NoSuchMethodException e) {
            return false;
        }
    }

    private static Path resolveQualifiedName(Path from, String qualifiedName) {
        return from.resolve(String.join("/", qualifiedName.split("\\.")) + ".java");
    }
}
