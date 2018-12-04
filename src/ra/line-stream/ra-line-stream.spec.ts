import * as fs from 'fs';
import { RaLineStream } from './ra-line-stream';
import { expect } from 'chai';
import { RaInputStream } from '../input-stream/ra-input-stream';
import { exampleLineOutput } from '../example.line-output.spec';


describe('RaLineStream', () => {
    let inputStream: RaInputStream;
    let lineStream: RaLineStream;
    let fileContent: string;

    before((done) => {
        fs.readFile('src/ra/example.ra', 'utf8', (err, c) => {
            if (err) throw err;
            fileContent = c;
            done();
        });
    });

    beforeEach((d) => {
        inputStream = new RaInputStream(fileContent);
        lineStream = new RaLineStream(inputStream);
        d();
    });

    it('should create stream', () => {
        expect(lineStream, 'stream created').to.exist;
    });

    it('should parse lines as expected', () => {
        let lines = [];
        while (!lineStream.eof()){
            lines.push(lineStream.next());
        }
        expect(lines.map(l => {
            const {
                _tokens,
                skipParsing,
                ...line
            } = l;

            return line;
        })).to.deep.equal(exampleLineOutput);

        expect(lines[0].span).to.be.equal(1);
    });

    it('should have parsed tokens', () => {
        let lines = [];
        while (!lineStream.eof()){
            lines.push(lineStream.next());
        }
        expect(lines[1]._tokens).to.exist;
    });

    it('should peek line', () => {
        expect(lineStream.peek().value).to.be.equal('`\n');
    });

    it('should peek line with shift', () => {
        expect(lineStream.peek(1).value).to.be.equal('\tthis is one string\n');
        expect(lineStream.peek(3).value).to.be.equal('\t`even now`\n');
    });

    it('should concatenate lines', () => {
        const line = lineStream.concatUntil((ln) => ln.end[0] === 3);
        expect(line.span).to.be.equal(3);
        expect(line.value).to.be.equal(exampleLineOutput.reduce((total, ln) => {
            if(ln.end[0] <= 3) {
                total+=ln._value;
            }
            return total;
        }, ''));
    });
});
