package prg.titech.api.responses;

import com.fasterxml.jackson.annotation.JsonProperty;
import prg.titech.chain.projection.ParseError;

import java.util.List;

public record AnalysisResponse(
        @JsonProperty("parse_errors") List<ParseError> parseErrors
) { }
