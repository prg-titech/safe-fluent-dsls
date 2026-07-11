package language;

import static language.BalancedParentheses.NonTerms.P;
import static language.BalancedParentheses.Alphabet.*;
import static fling.grammars.api.BNFAPI.bnf;

import fling.*;
import fling.BNF;
import fling.adapters.JavaMediator;

public class BalancedParentheses {
    public enum Alphabet implements Terminal {
        a, c
    }

    public enum NonTerms implements Variable {
        P
    }

    public static final BNF bnf = bnf(). //
            start(P). //
            derive(P).to(a, P, c, P). //
            derive(P).toEpsilon(). //
            build();
    public static final JavaMediator jm = new JavaMediator(bnf, //
            "example.generated", "BalancedParentheses", Alphabet.class);
}
