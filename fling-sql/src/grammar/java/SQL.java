import fling.BNF;
import fling.Terminal;
import fling.Variable;
import fling.adapters.JavaMediator;
import prg.titech.api.Grammar;

import static fling.grammars.api.BNFAPI.bnf;

@SuppressWarnings("unused") public class SQL implements Grammar {
    public enum Alphabet implements Terminal {
        select, from
    }

    public enum NonTerms implements Variable {
        Query
    }

    public static final BNF bnf = bnf(). //
            start(NonTerms.Query). //
            derive(NonTerms.Query).to(Alphabet.select.with(String.class), Alphabet.from.with(String.class)). //
            build();

    private static final JavaMediator jm = new JavaMediator(bnf, "", "SQL", Alphabet.class);

    @Override
    public JavaMediator getJavaMediator() {
        return jm;
    }
}
