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
badd +94 src/main.rs
badd +65 src/vim_commit.rs
badd +15 NOTES.md
badd +107 src/git_repository.rs
argglobal
%argdel
$argadd src/main.rs
$argadd src/vim_commit.rs
set stal=2
tabnew +setlocal\ bufhidden=wipe
tabrewind
edit src/main.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
wincmd _ | wincmd |
vsplit
2wincmd h
wincmd w
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
exe 'vert 1resize ' . ((&columns * 141 + 213) / 426)
exe 'vert 2resize ' . ((&columns * 141 + 213) / 426)
exe 'vert 3resize ' . ((&columns * 142 + 213) / 426)
argglobal
setlocal fdm=indent
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=99
setlocal fml=1
setlocal fdn=99
setlocal fen
63
normal! zo
74
normal! zo
93
normal! zo
107
normal! zo
let s:l = 94 - ((29 * winheight(0) + 26) / 53)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 94
normal! 014|
wincmd w
argglobal
2argu
if bufexists(fnamemodify("src/git_repository.rs", ":p")) | buffer src/git_repository.rs | else | edit src/git_repository.rs | endif
if &buftype ==# 'terminal'
  silent file src/git_repository.rs
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
let s:l = 107 - ((47 * winheight(0) + 26) / 53)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 107
normal! 0
wincmd w
argglobal
2argu
balt src/git_repository.rs
setlocal fdm=indent
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=99
setlocal fml=1
setlocal fdn=99
setlocal fen
22
normal! zo
23
normal! zo
27
normal! zo
34
normal! zo
42
normal! zo
53
normal! zo
66
normal! zo
let s:l = 65 - ((14 * winheight(0) + 26) / 53)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 65
normal! 012|
wincmd w
exe 'vert 1resize ' . ((&columns * 141 + 213) / 426)
exe 'vert 2resize ' . ((&columns * 141 + 213) / 426)
exe 'vert 3resize ' . ((&columns * 142 + 213) / 426)
tabnext
edit NOTES.md
argglobal
balt src/vim_commit.rs
setlocal fdm=syntax
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=99
setlocal fml=1
setlocal fdn=99
setlocal fen
2
normal! zo
let s:l = 26 - ((25 * winheight(0) + 26) / 53)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 26
normal! 0
tabnext 1
set stal=1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
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
