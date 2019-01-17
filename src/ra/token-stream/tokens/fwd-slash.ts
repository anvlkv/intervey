import { Token, TokenType } from '../token';
import { LineColumnAddress } from '../../line-column-address';


export class FWDSlashToken extends Token {
    constructor(start: LineColumnAddress, end: LineColumnAddress){
        super(
            TokenType.PUNCT,
            '/',
            start,
            end
        );
    }
}