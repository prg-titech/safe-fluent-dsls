package prg.titech;

import prg.titech.chain.Chain;
import prg.titech.chain.Call;
import prg.titech.sql.Query;
import prg.titech.sql.Where;


public class Main {

    public static void main(String[] args) {
        Chain query = Chain.builder()
                .call(Call.method("select").arg("*").build())
                .call(Call.method("from").arg("Students").build())
                .call(Call.method("where").arg(
                        Chain.builder()
                                .call(Call.method("columnId").arg("birth_year").build())
                                .call(Call.method("equals").build())
                                .call(Call.method("value").arg("2005").build())
                                .build()
                ))
                .build();
        System.out.println(query);

        Query simpleQuery = Query.select("*").from("Students").build();
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

        System.out.println(simpleQuery);
        System.out.println(complicatedQuery);
        System.out.println(subchainQuery);
    }
}
