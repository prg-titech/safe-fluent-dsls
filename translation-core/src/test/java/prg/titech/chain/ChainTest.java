package prg.titech.chain;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;

import static prg.titech.TestFixtures.*;

import java.util.stream.Stream;

public class ChainTest {
    
    protected static Stream<Arguments> chainExamples() {
        return Stream.of(
                Arguments.of(simpleQuery().toChain(), "select(\"*\").from(\"Students\").build()"),
                Arguments.of(complexQuery().toChain(), "select(\"name\", \"birth_year\").from(\"Students\").where(\"birth_year >= 2010 AND name = \\\"John Doe\\\" \").build()"),
                Arguments.of(subchainQuery().toChain(), "select(\"name\", \"birth_year\").from(\"Students\").where(\"birth_year < 2010  AND (name != \\\"Gary Stu\\\")\").build()")
        );
    }

    @ParameterizedTest
    @MethodSource("chainExamples")
    public void testPrettyPrint(Chain chain, String expectedResult) {
        Assertions.assertEquals(expectedResult, chain.toString());
    }
}
