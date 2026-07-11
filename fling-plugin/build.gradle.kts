plugins {
    id("java")
    `java-gradle-plugin`
}

group = "prg.titech"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation(gradleApi())
    testImplementation(platform("org.junit:junit-bom:6.0.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
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

tasks.test {
    useJUnitPlatform()
}