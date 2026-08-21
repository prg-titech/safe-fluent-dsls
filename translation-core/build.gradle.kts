plugins {
    id("java")
    id("application")
}

group = "prg.titech"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("jakarta.annotation:jakarta.annotation-api:3.0.0")
    implementation("com.github.jsqlparser:jsqlparser:5.3")
    implementation("com.github.javaparser:javaparser-symbol-solver-core:3.28.2")
    implementation("info.picocli:picocli:4.7.7")
    implementation("com.fasterxml.jackson.core:jackson-databind:2.22.1")
    testImplementation(platform("org.junit:junit-bom:6.0.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

application {
    // Define the main class for the application.
    mainClass = "prg.titech.api.BasicServer"
}

tasks.jar {
    manifest {
        attributes(
            "Main-Class" to application.mainClass
        )
    }
}

tasks.test {
    useJUnitPlatform()
}