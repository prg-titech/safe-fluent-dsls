package prg.titech;

import org.junit.jupiter.params.provider.Arguments;
import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.sql.Query;
import prg.titech.sql.Where;

import java.util.stream.Stream;

public class TestFixtures {

    public static Query simpleQuery() {
        return Query.select("*").from("Students").build();
    }

    public static Query complexQuery() {
        return Query.select("name", "birth_year")
                .from("Students")
                .where()
                .columnId("birth_year")
                .ge()
                .value("2010")
                .and()
                .columnId("name")
                .eq()
                .value("John Doe")
                .build();
    }

    public static Query subchainQuery() {
        return Query.select("name", "birth_year")
                .from("Students")
                .where(Where.columnId("birth_year").lt().value("2010").build())
                .and(Where.columnId("name").ne().value("Gary Stu").build())
                .build();
    }

    public static Query invalidSelectArgument() {
        return Query.select("name", "<")
                .from("Students")
                .build();
    }

    public static Query fromTwice() {
        return Query.select("*")
                .from("Students")
                .from("Teachers")
                .build();
    }

    public static Query wrongKeywordOrder() {
        return Query.select("*")
                .where()
                .columnId("name")
                .eq()
                .value("Mary Grey")
                .from("Students")
                .build();
    }

    public static Chain invalidSelectArgumentChain() {
        return Chain.builder()
                .call(Call.method("select").arg("= 1").build())
                .call(Call.method("from").arg("Students").build())
                .build();
    }

    public static Chain fromTwiceChain() {
        return Chain.builder()
                .call(Call.method("select").arg("*").build())
                .call(Call.method("from").arg("Students").build())
                .call(Call.method("from").arg("Teachers").build())
                .build();
    }

    public static Chain wrongKeywordOrderChain() {
        return Chain.builder()
                .call(Call.method("select").arg("*").build())
                .call(Call.method("from").arg("Students").build())
                .call(Call.method("where").build())
                .call(Call.method("eq").build())
                .call(Call.method("columnId").arg("age").build())
                .call(Call.method("value").arg("18").build())
                .build();
    }

    public static Stream<Arguments> invalidQueryChains() {
        return Stream.of(
                Arguments.of(invalidSelectArgumentChain()),
                Arguments.of(fromTwiceChain()),
                Arguments.of(wrongKeywordOrderChain())
        );
    }
}
