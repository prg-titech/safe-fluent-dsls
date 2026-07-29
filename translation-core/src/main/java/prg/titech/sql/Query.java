package prg.titech.sql;

import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import net.sf.jsqlparser.JSQLParserException;
import net.sf.jsqlparser.parser.CCJSqlParserUtil;
import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.builder.CallBuilder;
import prg.titech.chain.builder.ChainBuilder;

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

    public Chain toChain() {
        CallBuilder select = Call.method("select");
        for (String selection : selections) {
            select.arg(selection);
        }
        ChainBuilder result = Chain.builder()
                .call(select.build())
                .call(Call.method("from").arg(source));
        if (whereClause != null) {
            result.call(Call.method("where").arg(whereClause.raw()));
        }
        result.call(Call.method("build").build());
        return result.build();
    }
}
