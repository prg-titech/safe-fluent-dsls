import fling.BNF;
import fling.Terminal;
import fling.Variable;
import fling.adapters.JavaMediator;
import prg.titech.api.Grammar;

import static fling.grammars.api.BNFAPI.bnf;

public class Demo implements Grammar {
    public enum Alphabet implements Terminal {
        a, c
    }

    public enum NonTerms implements Variable {
        P
    }

    public static final BNF bnf = bnf(). //
            start(NonTerms.P). //
            derive(NonTerms.P).to(Alphabet.a, NonTerms.P, Alphabet.c, NonTerms.P). //
            derive(NonTerms.P).toEpsilon(). //
            build();
    public static final JavaMediator jm = new JavaMediator(bnf, //
            "example.generated", "BalancedParentheses", Alphabet.class);

    @Override
    public JavaMediator getJavaMediator() {
        return jm;
    }
}
