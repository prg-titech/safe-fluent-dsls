package prg.titech.api;

import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpHandler;
import com.sun.net.httpserver.HttpServer;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.util.Optional;

public class BasicServer {
    public static void main(String[] args) throws Exception {
        Options options = parseOptions(args);
        HttpServer server = start(options.port().orElse(8080));
        System.out.println("BACKEND_PORT " + server.getAddress().getPort());
    }

    public static HttpServer start(int port) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress(port), 0);
        server.createContext("/health", new HealthHandler());
        server.createContext("/analyze", new AnalyzeHandler());
        server.setExecutor(null); // default executor
        server.start();
        return server;
    }

    private static Options parseOptions(String[] args) {
        Optional<Integer> port = Optional.empty();
        // If the last entry in arg is "--port", then the actual port _number_ is unspecified!
        for (int i = 0; i < args.length - 1; i++) {
            if (args[i].equals("-p") || args[i].equals("--port")) {
                port = Optional.of(Integer.parseInt(args[i + 1]));
            }
        }
        return new Options(port);
    }

    private record Options(Optional<Integer> port) { }

    static class HealthHandler implements HttpHandler {
        public void handle(HttpExchange exchange) {
            try {
                exchange.sendResponseHeaders(200, -1);
            } catch (Exception e) {
                e.printStackTrace(System.err);
            }
        }
    }
}