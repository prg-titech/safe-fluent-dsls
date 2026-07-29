package prg.titech.sql;

import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import net.sf.jsqlparser.JSQLParserException;
import net.sf.jsqlparser.parser.CCJSqlParserUtil;

public record Query(
        String[] selections,
        String source,
        @Nullable Where whereClause
) {
    public Query(String[] selections, String source) {
        this(selections, source, null);
    }

    public static QueryBuilder select(String... selections) {
        return new QueryBuilder(selections);
    }

    public boolean isValid() {
        try {
            CCJSqlParserUtil.parse(toString());
            return true;
        } catch (JSQLParserException e) {
            System.err.printf("Encountered error during Query parse: %s%n", e);
            e.printStackTrace(System.err);
            return false;
        }
    }

    @Override
    @Nonnull
    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append("SELECT ");
        sb.append(String.join(", ", selections));
        sb.append(" FROM ");
        sb.append(source);
        if (whereClause != null) {
            sb.append(" WHERE ");
            sb.append(whereClause.raw());
        }
        return sb.toString();
    }
}
