package prg.titech.sql.analyze;

import net.sf.jsqlparser.JSQLParserException;
import net.sf.jsqlparser.parser.CCJSqlParserUtil;
import net.sf.jsqlparser.parser.ParseException;

import java.util.Arrays;

public class SQLAnalyzer {

    public static void parse(String rawSQL) {
        ParseException error;
        try {
            CCJSqlParserUtil.parse(rawSQL);
            return;
        } catch (JSQLParserException e) {
            if (e.getCause().getCause() instanceof ParseException pe) {
                error = pe;
            } else {
                return;
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
        System.out.println("Expected tokens: " + Arrays.deepToString(error.expectedTokenSequences));
    }
}
