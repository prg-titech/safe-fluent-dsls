package prg.titech.sql.analyze;

import net.sf.jsqlparser.JSQLParserException;
import net.sf.jsqlparser.parser.CCJSqlParserUtil;
import net.sf.jsqlparser.parser.ParseException;
import prg.titech.chain.translate.TokenList;

import java.util.Arrays;

public class SQLAnalyzer {

    public static boolean analyze(TokenList tokens) {
        String rawSQL = tokens.toString();
        System.out.println(rawSQL);
        ParseException error;
        try {
            CCJSqlParserUtil.parse(rawSQL);
            return true;
        } catch (JSQLParserException e) {
            if (e.getCause().getCause() instanceof ParseException pe) {
                error = pe;
            } else {
                return true;
            }
        }

        System.out.println("Current Token: " + error.currentToken);
        System.out.println("\timage: " + error.currentToken.image);
        System.out.println("\tbegin: " + error.currentToken.absoluteBegin);
        System.out.println("\tend: " + error.currentToken.absoluteEnd);
        System.out.println("\tbeginLine: " + error.currentToken.beginLine);
        System.out.println("\tbeginColumn: " + error.currentToken.beginColumn);
        System.out.println("\tendLine: " + error.currentToken.endLine);
        System.out.println("\tendColumn: " + error.currentToken.endColumn);
        System.out.println("Token Image: " + Arrays.toString(error.tokenImage));
        System.out.println("Expected tokens: " + Arrays.stream(error.expectedTokenSequences).map(seq -> Arrays.stream(seq).mapToObj(token -> error.tokenImage[token]).toList()).toList());
        return false;
    }
}
