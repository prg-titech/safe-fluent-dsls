package prg.titech;

import prg.titech.sql.Query;
import prg.titech.sql.Where;

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
}
