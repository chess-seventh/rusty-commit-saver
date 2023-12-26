let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd ~/src/git.sr.ht/~chess7th/rusty-commit-saver
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +11 src/main.rs
badd +90 src/vim_commit.rs
argglobal
%argdel
$argadd src/main.rs
edit src/vim_commit.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 212 + 213) / 426)
exe 'vert 2resize ' . ((&columns * 213 + 213) / 426)
argglobal
balt src/main.rs
setlocal fdm=indent
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=99
setlocal fml=1
setlocal fdn=99
setlocal fen
21
normal! zo
22
normal! zo
23
normal! zo
40
normal! zo
51
normal! zo
67
normal! zo
81
normal! zo
93
normal! zo
let s:l = 86 - ((2 * winheight(0) + 20) / 41)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 86
normal! 014|
wincmd w
argglobal
if bufexists(fnamemodify("src/main.rs", ":p")) | buffer src/main.rs | else | edit src/main.rs | endif
if &buftype ==# 'terminal'
  silent file src/main.rs
endif
balt src/vim_commit.rs
setlocal fdm=indent
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=99
setlocal fml=1
setlocal fdn=99
setlocal fen
52
normal! zo
52
normal! zo
63
normal! zo
75
normal! zo
86
normal! zo
let s:l = 11 - ((2 * winheight(0) + 20) / 41)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 11
normal! 010|
lcd ~/src/git.sr.ht/~chess7th/rusty-commit-saver/src
wincmd w
2wincmd w
exe 'vert 1resize ' . ((&columns * 212 + 213) / 426)
exe 'vert 2resize ' . ((&columns * 213 + 213) / 426)
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
set hlsearch
nohlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
