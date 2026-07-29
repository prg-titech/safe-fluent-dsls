package prg.titech.sql;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;

import java.util.stream.Stream;

public class QueryTest {

    protected static Stream<Arguments> validQueries() {
        return Stream.of(
                Arguments.of(Query.select("*").from("Students").build()),
                Arguments.of(Query.select("name", "birth_year")
                        .from("Students")
                        .where()
                        .columnId("birth_year")
                        .ge()
                        .value("2010")
                        .and()
                        .columnId("name")
                        .eq()
                        .value("John Doe")
                        .build()),
                Arguments.of(Query.select("name", "birth_year")
                        .from("Students")
                        .where(Where.columnId("birth_year").lt().value("2010").build())
                        .and(Where.columnId("name").ne().value("Gary Stu").build())
                        .build())
        );
    }

    @ParameterizedTest
    @MethodSource("validQueries")
    @DisplayName("testIsValid")
    public void testValid(Query query) {
        Assertions.assertTrue(query.isValid());
    }


}
