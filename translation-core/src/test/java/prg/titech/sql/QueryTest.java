package prg.titech.sql;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;

import java.util.stream.Stream;

import static prg.titech.TestFixtures.*;

public class QueryTest {

    protected static Stream<Arguments> validQueries() {
        return Stream.of(
                Arguments.of(simpleQuery()),
                Arguments.of(complexQuery()),
                Arguments.of(subchainQuery())
        );
    }

    @ParameterizedTest
    @MethodSource("validQueries")
    @DisplayName("testIsValid")
    public void testValid(Query query) {
        Assertions.assertTrue(query.isValid());
    }


}
