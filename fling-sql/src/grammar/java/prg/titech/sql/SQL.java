package prg.titech.sql;

import fling.BNF;
import fling.Terminal;
import fling.Variable;
import fling.adapters.JavaMediator;
import prg.titech.api.Grammar;

import static fling.grammars.api.BNFAPI.bnf;

@SuppressWarnings("unused") public class SQL implements Grammar {
    public enum Alphabet implements Terminal {
        select, from, where, columnId, equals, nat
    }

    public enum NonTerms implements Variable {
        Query, Where, Exp
    }

    public static final BNF bnf = bnf(). //
            start(NonTerms.Query). //
            derive(NonTerms.Query).to(Alphabet.select.with(String.class), Alphabet.from.with(String.class), NonTerms.Where).
            derive(NonTerms.Where).toEpsilon().
            derive(NonTerms.Where).to(Alphabet.where.with(String.class)).
            derive(NonTerms.Where).to(Alphabet.where.with(NonTerms.Exp)).
            derive(NonTerms.Exp).to(Alphabet.columnId.with(String.class), Alphabet.equals, Alphabet.nat.with(Integer.class)).
            build();

    private static final JavaMediator jm = new JavaMediator(bnf, "prg.titech.sql.generated", "SQL", Alphabet.class);

    @Override
    public JavaMediator getJavaMediator() {
        return jm;
    }
}
