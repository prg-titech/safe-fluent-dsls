package prg.titech;

import prg.titech.sql.Query;
import prg.titech.sql.Where;
import prg.titech.sql.analyze.SQLAnalyzer;
import prg.titech.sql.translate.Translator;


public class Main {

    public static void main(String[] args) {
        Query simpleQuery = Query.select("wow =% 1").from("Students").build();
        Query complicatedQuery = Query.select("name", "birth_year")
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

        Query subchainQuery = Query.select("name", "birth_year")
                .from("Students")
                .where(Where.columnId("birth_year").lt().value("2010").build())
                .and(Where.columnId("name").ne().value("Gary Stu").build())
                .build();

        SQLAnalyzer.parse(Translator.translate(simpleQuery.toChain()));
        SQLAnalyzer.parse(Translator.translate(complicatedQuery.toChain()));
        SQLAnalyzer.parse(Translator.translate(subchainQuery.toChain()));
    }
}
