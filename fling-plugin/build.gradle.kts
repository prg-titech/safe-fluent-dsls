plugins {
    id("java")
    `java-gradle-plugin`
    `maven-publish`
}

group = "prg.titech"
version = "0.1-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation(gradleApi())
    implementation(files("lib/fling-1.0.0.jar"))
    implementation("com.google.googlejavaformat:google-java-format:1.6")
    testImplementation(platform("org.junit:junit-bom:6.0.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testImplementation("commons-io:commons-io:2.22.0")
    testImplementation(gradleTestKit())
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

gradlePlugin {
    plugins {
        create("fling-plugin") {
            id = "prg.titech.fling-plugin"
            implementationClass = "prg.titech.FlingPlugin"
        }
    }
}

publishing {
    publications {
        create<MavenPublication>("api") {
            artifactId = "fling-api"
            from(components["java"])
        }
    }
}

tasks.jar {
    from(zipTree("lib/fling-1.0.0.jar"))
}

tasks.test {
    dependsOn(tasks.jar)
    useJUnitPlatform()
}