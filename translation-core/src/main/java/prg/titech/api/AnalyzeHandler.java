package prg.titech.api;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpHandler;
import prg.titech.api.requests.AnalysisRequest;
import prg.titech.api.responses.AnalysisResponse;
import prg.titech.chain.Chain;
import prg.titech.chain.find.java.JavaChainSearcher;
import prg.titech.chain.projection.ParseError;
import prg.titech.sql.analyze.SQLAnalyzer;
import prg.titech.sql.translate.SQLTranslator;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Optional;
import java.util.Set;

public class AnalyzeHandler implements HttpHandler {
    @Override
    public void handle(HttpExchange exchange) throws IOException {
        Optional<AnalysisRequest> optRequest = checkPostRequest(exchange);
        if (optRequest.isEmpty()) {
            exchange.sendResponseHeaders(400, -1);
            return;
        }

        AnalysisRequest request = optRequest.get();
        AnalysisResponse response = analyze(request);
        byte[] rawResponse;
        try {
            rawResponse = JsonUtil.writeJson(response);
        } catch (JsonProcessingException e) {
            e.printStackTrace(System.err);
            return;
        }

        exchange.sendResponseHeaders(200, rawResponse.length);
        exchange.getResponseHeaders().add("Content-type", "application/json");
        OutputStream os = exchange.getResponseBody();
        os.write(rawResponse);
        os.close();
    }

    private Optional<AnalysisRequest> checkPostRequest(HttpExchange exchange) throws IOException {
        if (!exchange.getRequestMethod().equals("POST")) {
            return Optional.empty();
        }
        InputStream is = exchange.getRequestBody();
        byte[] rawRequest = is.readAllBytes();
        AnalysisRequest request;
        try {
            String rawRequestString = new String(rawRequest, StandardCharsets.UTF_8);
            System.out.println(rawRequestString);
            request = JsonUtil.readJson(rawRequestString, AnalysisRequest.class);
        } catch (JsonProcessingException e) {
            e.printStackTrace(System.err);
            return Optional.empty();
        }
        return Optional.of(request);
    }

    private AnalysisResponse analyze(AnalysisRequest request) {
        List<Chain> chains = JavaChainSearcher.findChains(request.rawSourceFile(), Set.of("select"));
        List<ParseError> parseErrors = chains.stream()
                .map(SQLTranslator::translate)
                .flatMap(t -> SQLAnalyzer.parse(t).stream())
                .toList();
        return new AnalysisResponse(parseErrors);
    }

}
