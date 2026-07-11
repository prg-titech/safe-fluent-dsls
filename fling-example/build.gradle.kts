plugins {
    id("java")
    id("application")
}

var javaMainClass = "org.example.Main"

group = "org.example"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

sourceSets {
    create("grammar") {
        java {}
    }
}

val flingConfiguration: Configuration = configurations.create("flingConfiguration")

configurations.named("grammarImplementation") {
    extendsFrom(flingConfiguration)
}

configurations.implementation {
    extendsFrom(flingConfiguration)
}

dependencies {
    flingConfiguration(files("lib/fling-1.0.0.jar"))
    "grammarImplementation"("com.google.googlejavaformat:google-java-format:1.35.0")
    testImplementation(platform("org.junit:junit-bom:6.0.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

sourceSets.main.configure {
    compileClasspath += sourceSets["grammar"].output
    runtimeClasspath += sourceSets["grammar"].output
}

tasks.test {
    useJUnitPlatform()
}

tasks.register<JavaExec>("generate") {
    group = "Generate"
    description = "Generates the example languages"

    mainClass = "language.Generate"
    classpath = sourceSets["grammar"].runtimeClasspath

    // Google Format needs these arguments to work with JDK 16+
    // See: https://github.com/google/google-java-format#jdk-16
    jvmArgs(
        "--add-exports=jdk.compiler/com.sun.tools.javac.api=ALL-UNNAMED",
            "--add-exports=jdk.compiler/com.sun.tools.javac.code=ALL-UNNAMED",
            "--add-exports=jdk.compiler/com.sun.tools.javac.file=ALL-UNNAMED",
            "--add-exports=jdk.compiler/com.sun.tools.javac.parser=ALL-UNNAMED",
            "--add-exports=jdk.compiler/com.sun.tools.javac.tree=ALL-UNNAMED",
            "--add-exports=jdk.compiler/com.sun.tools.javac.util=ALL-UNNAMED"
    )
}

tasks.register<Delete>("cleanGenerated") {
    group = "Build"
    description = "Cleans the generated fluent APIs"

    delete.add("src/main/java/example/generated")
}

tasks.compileJava.configure {
    dependsOn("generate")
}

tasks.run.configure {
    mainClass = javaMainClass
}

tasks.clean.configure {
    dependsOn("cleanGenerated")
}

