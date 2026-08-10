package prg.titech.sql.util;

import prg.titech.chain.token.Position;
import prg.titech.chain.token.Range;
import prg.titech.chain.token.Token;

public class TokenUtil {

    public static Token fromSqlToken(net.sf.jsqlparser.parser.Token source) {
        String image = source.image;
        Position begin = new Position(source.beginLine, source.beginColumn);
        Position end = new Position(source.endLine, source.endColumn);
        Range range = new Range(begin, end);
        return new Token(image, range);
    }
}
