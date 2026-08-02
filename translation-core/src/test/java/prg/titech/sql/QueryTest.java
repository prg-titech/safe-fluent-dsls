package prg.titech.sql;

import net.sf.jsqlparser.parser.CCJSqlParserUtil;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;
import prg.titech.chain.translate.TokenList;
import prg.titech.sql.translate.SQLTranslator;

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
    public void testValid(Query query) {
        Assertions.assertTrue(query.isValid());
    }

    @ParameterizedTest
    @MethodSource("validQueries")
    public void testTranslation(Query query) {
        TokenList translation = SQLTranslator.translateTokens(query.toChain());
        System.out.println(translation.toDebugString());
        Assertions.assertDoesNotThrow(() -> CCJSqlParserUtil.parse(translation.toString()));
    }


}
