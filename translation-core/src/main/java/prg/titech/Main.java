package prg.titech;

import prg.titech.chain.Chain;
import prg.titech.chain.Call;


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
    }
}
