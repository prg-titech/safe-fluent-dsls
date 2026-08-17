package prg.titech.api;

import com.sun.net.httpserver.HttpServer;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.FieldSource;
import prg.titech.api.requests.AnalysisRequest;
import prg.titech.api.responses.AnalysisResponse;

import java.io.BufferedReader;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.stream.Collectors;

public class ApiTest {
    private static final Path[] testPaths = new Path[] { Paths.get("src", "test", "java", "prg", "titech", "TestFixtures.java") };
    private static HttpServer backend;

    @BeforeAll
    public static void startServer() throws IOException {
        backend = BasicServer.start(8080);
    }

    @AfterAll
    public static void stopServer() {
        backend.stop(0);
    }

    @ParameterizedTest
    @FieldSource("testPaths")
    public void testAnalyze(Path sourceFile) throws IOException, InterruptedException {
        String source = Files.readString(sourceFile);
        AnalysisRequest request = new AnalysisRequest(source, "java", "sql", "prg.titech.sql.Query");
        Process curl = Runtime.getRuntime().exec(
                new String[] {"curl", "-XPOST", "localhost:8080/analyze", "-d", JsonUtil.writeJsonString(request)}
        );
        Assertions.assertEquals(0, curl.waitFor());
        AnalysisResponse response;
        try (BufferedReader reader = curl.inputReader()) {
            String rawResponse = reader.lines().collect(Collectors.joining("\n"));
            response = JsonUtil.readJson(rawResponse, AnalysisResponse.class);
        }
        System.out.println(response.parseErrors());
    }
}
