package prg.titech.sql.analyze;

import net.sf.jsqlparser.JSQLParserException;
import net.sf.jsqlparser.parser.CCJSqlParserUtil;
import net.sf.jsqlparser.parser.ParseException;
import prg.titech.chain.projection.ParseError;
import prg.titech.chain.token.Token;
import prg.titech.chain.translate.Translation;
import prg.titech.sql.util.TokenUtil;

import java.util.List;
import java.util.Optional;

public class SQLAnalyzer {

    public static Optional<ParseException> analyze(String rawSql) {
        try {
            CCJSqlParserUtil.parse(rawSql);
            return Optional.empty();
        } catch (JSQLParserException e) {
            if (e.getCause().getCause() instanceof ParseException pe) {
                return Optional.of(pe);
            } else {
                return Optional.empty();
            }
        }
    }

    public static List<ParseError> parse(Translation translation) {
        return analyze(translation.toString()).stream().map(p -> {
            Token targetToken = TokenUtil.fromSqlToken(p.currentToken.next);
            System.out.println("Parse Error occurred when consuming: " + targetToken.toDebugString());
            List<Token> sourceTokens = translation.findSourceOfOrAfter(targetToken);
            System.out.println("Source tokens identified: " + sourceTokens.stream().map(Token::toDebugString).toList());
            return new ParseError(sourceTokens.getFirst(), targetToken, p.getMessage());
        }).toList();
    }
}
