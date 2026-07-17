plugins {
    id("java")
    id("prg.titech.fling-plugin") version "0.1-SNAPSHOT"
}

group = "prg.titech"
version = "1.0-SNAPSHOT"

repositories {
    mavenLocal()
    mavenCentral()
}

dependencies {
    "grammarImplementation"("prg.titech:fling-api:0.1-SNAPSHOT")
    implementation("prg.titech:fling-api:0.1-SNAPSHOT")
    testImplementation(platform("org.junit:junit-bom:6.0.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
}