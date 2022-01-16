import React from "react";
import ReactDOM from "react-dom";

import { Chip8Component } from '../build/chip8_wasm';

const Chip8 = (props) => {
    const ref = React.useRef()
    const scale = props.scale || 12;
    let chip8;

    React.useEffect(() => {
        let ctx = ref.current.getContext('2d')
        chip8 = new Chip8Component(ctx, scale, []);
    }, [])

    const keydown = (e) => {
        chip8.key_down(e.keyCode);
    }

    const keyup = (e) => {
        chip8.key_up(e.keyCode);
    }

    const start = () => {
        const tick = () => {
            chip8.tick();
            window.requestAnimationFrame(() => tick());
        }

        window.requestAnimationFrame(tick);
    }

    const onFileInput = (event) => {
        const files = event.target.files;
        for (let i = 0; i < Math.min(files.length, 1); ++i) {
            let file = files.item(i);
            file.arrayBuffer().then((buffer) => {
                let rom = new Uint8Array(buffer);
                chip8.load(rom);
                start();
            })
        }
    }

    // return (
    //     <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
    //         <canvas id="canvas" width={64 * scale} height={32 * scale} ref={ref} tabIndex={0} onKeyDown={keydown} onKeyUp={keyup} />
    //         <input type="file" onChange={onFileInput} accept={".bin, .ch8"} />
    //     </div>
    // )

    return <canvas id="canvas" width={64 * scale} height={32 * scale} ref={ref} tabIndex={0} onKeyDown={keydown} onKeyUp={keyup} />
}

// ReactDOM.render(<Chip8 />, document.getElementById("root"));

exports.Chip8 = Chip8;
