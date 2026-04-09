p201p91+10p94+11p220p53*21p91+30pv
      v                          <
v     <_v>
 >v>v>|`3
 01g+`2g0                                    this is a test
 +g00g00g             
 gg111g01               this is a test
 00gg00g0             
 100pg10g                          this is a test
0gg001p3+
001p120p3
g00ggp000
0gg111g0p
1g-0+01+>^
>^>^>^>^

00=cur x (0)
01=cur y (2)
10=width (10)
11=height (13)
20=y reset value (2)
21=y upper value (15)
30=x upper value (10)
1. load char a from cx,cy
2. load char b from cx+w,cy
3. store char b at cx-w,cy
4. store char a at cx+w,cy
5. cy++
6. if cy<yu goto 1
7. cy=yr
8. cx++
9. if cx<xu goto 1
10. xu+=w
11. goto 1, but in next copy
