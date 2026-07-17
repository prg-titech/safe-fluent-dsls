package prg.titech.sql;

import java.util.Arrays;
import java.util.List;
import java.util.Optional;

import static prg.titech.sql.generated.SQL.*;
import static prg.titech.sql.generated.SQLAST.*;

public class QueryWrapper {
    List<String> selections;
    String from;
    Optional<Where> whereClause;

    public static void main(String[] args) {
        Query ast1 = select("*").from("Students").$();
        System.out.println(ast1.select);

         Query ast2 = select("name, birth_year").from("Students")
                .where(Exp.columnId("birth_year").equals().nat(2005)).$();

        Query ast3 = select("name, birth_year").from("Students")
                .where("birth_year == 2005").$();
    }

    public QueryWrapper(List<String> selections, String from, Optional<Where> whereClause) {
        this.selections = selections;
        this.from = from;
        this.whereClause = whereClause;
    }

    public static QueryWrapper from(Query query) {
        List<String> selections = Arrays.stream(query.select.split(",")).map(String::strip).toList();
        String from = query.from;
        Optional<Where> whereClause = Optional.ofNullable(query.where);
        return new QueryWrapper(selections, from, whereClause);
    }

    @Override
    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append("SELECT ");
        sb.append(String.join(", ", this.selections));
        sb.append(" FROM ");
        sb.append(this.from);
        if (whereClause.isPresent()) {
            sb.append(" WHERE ");
            sb.append(whereClause.get());
        }
        return sb.toString();
    }
}
