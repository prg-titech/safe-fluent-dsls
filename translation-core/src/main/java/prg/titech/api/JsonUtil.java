package prg.titech.api;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;

public class JsonUtil {
    private static final ObjectMapper objectMapper = new ObjectMapper();

    public static <T> T readJson(String rawJson, Class<T> clazz) throws JsonProcessingException {
        return objectMapper.readValue(rawJson, clazz);
    }

    public static <T> byte[] writeJson(T value) throws JsonProcessingException {
        return objectMapper.writeValueAsBytes(value);
    }

    public static <T> String writeJsonString(T value) throws JsonProcessingException {
        return objectMapper.writeValueAsString(value);
    }
}
