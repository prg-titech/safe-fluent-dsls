package prg.titech.api.requests;

import com.fasterxml.jackson.annotation.JsonProperty;

public record AnalysisRequest(
        @JsonProperty("raw_source_file") String rawSourceFile,
        @JsonProperty("host_language") String hostLanguage,
        @JsonProperty("embedded_language") String embeddedLanguage,
        @JsonProperty("fluent_api") String fluentApi
) { }
